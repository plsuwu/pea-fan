use std::collections::HashMap;
use std::sync::atomic::AtomicUsize;
use std::time::{Duration, Instant};
use std::{collections::HashSet, sync::Arc};

use thiserror::Error;
use tokio::sync::mpsc::{self, UnboundedReceiver};
use tokio::sync::{Mutex, RwLock, broadcast, oneshot};
use tokio::task::JoinHandle;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::database::schema::ChannelBasic;
use crate::socket;
use crate::socket::client::{IrcClient, IrcClientConfig, IrcResult};
use crate::socket::core::{IrcError, IrcEvent};
use crate::util::channel;

pub const DEFAULT_CAPS: &str = "CAP REQ :twitch.tv/tags twitch.tv/commands";
pub const DEFAULT_IRC: &str = "wss://irc-ws.chat.twitch.tv/";

pub type SocketPoolResult<T> = core::result::Result<T, SocketPoolError>;

#[derive(Error, Debug)]
pub enum SocketPoolError {
    #[error("all clients full - no clients available to handle join request")]
    NoJoinSlots,

    #[error("reqwest error: {0}")]
    ReqwestError(#[from] reqwest::Error),

    #[error("channel util error: {0}")]
    ChannelUtilError(#[from] channel::ChannelUtilError),

    #[error("irc websocket client error: {0}")]
    IrcClientError(#[from] socket::core::IrcError),
}

#[derive(Debug, Clone)]
pub struct PoolConfig {
    pub base_config: IrcClientConfig,
    pub max_per_connection: usize,
    pub max_connections: usize,
    pub min_connections: usize,
    pub timeout: Duration,
    pub health_check_interval: Duration,
    pub scale_up_threshold: f64,
    pub scale_down_threshold: f64,
    pub rebalance_interval: Duration,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            base_config: IrcClientConfig::default(),
            max_per_connection: 100,
            max_connections: 3,
            min_connections: 1,
            timeout: Duration::from_secs(30),
            health_check_interval: Duration::from_secs(60),
            scale_up_threshold: 0.8,
            scale_down_threshold: 0.3,
            rebalance_interval: Duration::from_secs(300),
        }
    }
}

#[derive(Debug)]
pub enum PoolEvent {
    ConnectionEvent {
        connection_id: String,
        event: IrcEvent,
    },
    ScaleUp,
    ScaleDown,
    Rebalance,
    CheckHealth,
}

#[derive(Debug)]
pub enum PoolCommand {
    JoinChannel {
        channel: String,
        response: oneshot::Sender<IrcResult<()>>,
    },
    LeaveChannel {
        channel: String,
        response: oneshot::Sender<IrcResult<()>>,
    },
    SendMessage {
        channel: String,
        message: String,
        response: oneshot::Sender<IrcResult<()>>,
    },
    GetStats {
        response: oneshot::Sender<PoolStats>,
    },
    Shutdown {
        response: oneshot::Sender<()>,
    },
}

#[derive(Debug, Clone)]
pub enum BalancingStrategy {
    RoundRobin,
    LeastLoaded,
    Random,
}

#[derive(Debug, Clone)]
pub struct ConnectionStats {
    pub id: String,
    pub channels: Vec<String>,
    pub channel_count: usize,
    pub is_connected: bool,
    pub created_at: Instant,
    pub last_activity: Instant,
    pub processed: u64,
    pub errors: u64,
}

#[derive(Debug, Clone)]
pub struct PoolStats {
    pub total_connections: usize,
    pub active_connections: usize,
    pub total_channels: usize,
    pub average_load: f64,
    pub connections: Vec<ConnectionStats>,
}

#[derive(Debug)]
pub struct PooledConnection {
    pub id: String,
    pub client: IrcClient,
    pub channels: Arc<RwLock<HashSet<String>>>,
    pub is_connected: Arc<RwLock<bool>>,
    pub created_at: Instant,
    pub last_activity: Arc<RwLock<Instant>>,
    pub processed: Arc<AtomicUsize>,
    pub errors: Arc<AtomicUsize>,
    pub event_handle: Option<JoinHandle<()>>,
}

impl PooledConnection {
    pub async fn new(
        config: IrcClientConfig,
        pool_tx: mpsc::UnboundedSender<PoolEvent>,
    ) -> IrcResult<Self> {
        let id = uuid::Uuid::new_v4().to_string();
        let (client, event_rx) = IrcClient::new(config);

        let channels = Arc::new(RwLock::new(HashSet::new()));
        let is_connected = Arc::new(RwLock::new(false));
        let last_activity = Arc::new(RwLock::new(Instant::now()));
        let processed = Arc::new(AtomicUsize::new(0));
        let errors = Arc::new(AtomicUsize::new(0));

        let event_handle = {
            let conn_id = id.clone();
            let chan_clone = channels.clone();
            let is_connected_clone = is_connected.clone();
            let last_activity_clone = last_activity.clone();
            let processed_clone = processed.clone();
            let errors_clone = errors.clone();

            tokio::spawn(Self::handle_events(
                conn_id,
                event_rx,
                pool_tx,
                chan_clone,
                is_connected_clone,
                last_activity_clone,
                processed_clone,
                errors_clone,
            ))
        };

        Ok(Self {
            id,
            client,
            channels,
            is_connected,
            created_at: Instant::now(),
            last_activity,
            processed,
            errors,
            event_handle: Some(event_handle),
        })
    }

    async fn handle_events(
        connection_id: String,
        mut event_rx: mpsc::UnboundedReceiver<IrcEvent>,
        pool_tx: mpsc::UnboundedSender<PoolEvent>,
        channels: Arc<RwLock<HashSet<String>>>,
        is_connected: Arc<RwLock<bool>>,
        last_activity: Arc<RwLock<Instant>>,
        processed: Arc<AtomicUsize>,
        errors: Arc<AtomicUsize>,
    ) {
        while let Some(event) = event_rx.recv().await {
            *last_activity.write().await = Instant::now();
            processed.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

            match &event {
                IrcEvent::Connected => {
                    *is_connected.write().await = true;
                    info!("connection {}: established", connection_id);
                }
                IrcEvent::Disconnected => {
                    *is_connected.write().await = false;
                    warn!("connection {}: disconnected", connection_id);
                }

                // IrcEvent::ChannelJoined(channel) => todo!(),
                // IrcEvent::ChannelParted(channel) => todo!(),
                IrcEvent::Error(irc_error) => {
                    error!("connection {}: error: {:?}", connection_id, irc_error);
                    errors.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                }
                _ => {}
            }

            let pool_event = PoolEvent::ConnectionEvent {
                connection_id: connection_id.clone(),
                event,
            };

            if let Err(_) = pool_tx.send(pool_event) {
                warn!("connection {}: pool event channel closed", connection_id);
                break;
            }
        }

        info!("connection {}: event handler terminated", connection_id);
    }

    async fn connect(&mut self) -> IrcResult<()> {
        self.client.connect().await
    }

    async fn join_channel(&self, channel: &str) -> IrcResult<()> {
        let res = self.client.join_channel(channel).await;
        if res.is_ok() {
            self.channels.write().await.insert(channel.to_string());
        }

        res
    }

    async fn leave_channel(&self, channel: &str) -> IrcResult<()> {
        let res = self.client.leave_channel(channel).await;
        if res.is_ok() {
            self.channels.write().await.remove(channel);
        }

        res
    }

    async fn send_message(&self, channel: &str, message: &str) -> IrcResult<()> {
        self.client.send_message(channel, message).await
    }

    async fn channel_count(&self) -> usize {
        self.channels.read().await.len()
    }

    async fn has_channel(&self, channel: &str) -> bool {
        self.channels.read().await.contains(channel)
    }

    async fn get_stats(&self) -> ConnectionStats {
        let channels: Vec<String> = self.channels.read().await.iter().cloned().collect();

        ConnectionStats {
            id: self.id.clone(),
            channel_count: channels.len(),
            channels,
            is_connected: *self.is_connected.read().await,
            created_at: self.created_at,
            last_activity: *self.last_activity.read().await,
            processed: self.processed.load(std::sync::atomic::Ordering::Relaxed) as u64,
            errors: self.errors.load(std::sync::atomic::Ordering::Relaxed) as u64,
        }
    }
}

impl Drop for PooledConnection {
    fn drop(&mut self) {
        if let Some(handle) = self.event_handle.take() {
            handle.abort();
        }
    }
}

#[derive(Clone)]
/// Primary IRC pool
pub struct IrcConnectionPool {
    pub config: PoolConfig,
    pub connections: Arc<RwLock<HashMap<String, Arc<PooledConnection>>>>,
    pub channel_map: Arc<RwLock<HashMap<String, String>>>, // HashMap<channel, connection_id>
    pub command_tx: mpsc::UnboundedSender<PoolCommand>,
    pub event_broadcast: broadcast::Sender<IrcEvent>,
    pub load_balancing: BalancingStrategy,
    pub next_connection_index: Arc<AtomicUsize>,
}

impl IrcConnectionPool {
    pub fn new(config: PoolConfig) -> (Self, broadcast::Receiver<IrcEvent>) {
        let (command_tx, _) = mpsc::unbounded_channel();
        let (event_broadcast, event_rx) = broadcast::channel(1000);

        let pool = Self {
            config,
            connections: Arc::new(RwLock::new(HashMap::new())),
            channel_map: Arc::new(RwLock::new(HashMap::new())),
            command_tx,
            event_broadcast,
            load_balancing: BalancingStrategy::LeastLoaded,
            next_connection_index: Arc::new(AtomicUsize::new(0)),
        };

        (pool, event_rx)
    }

    pub async fn start(&mut self) -> IrcResult<()> {
        let (command_tx, command_rx) = mpsc::unbounded_channel();
        self.command_tx = command_tx;

        self.ensure_min_connections().await?;

        let pool_manager = PoolManager::new(
            self.config.clone(),
            self.connections.clone(),
            self.channel_map.clone(),
            self.event_broadcast.clone(),
            self.load_balancing.clone(),
            self.next_connection_index.clone(),
        );

        tokio::spawn(pool_manager.run(command_rx));

        self.start_healthcheck().await;
        self.start_rebalancer().await;

        info!(
            "irc websocket connection pool started ({} initial connections)",
            self.config.min_connections
        );
        Ok(())
    }

    async fn ensure_min_connections(&self) -> IrcResult<()> {
        let mut connections = self.connections.write().await;
        let (pool_tx, mut pool_rx) = mpsc::unbounded_channel();

        for _ in 0..self.config.min_connections {
            let mut connection =
                PooledConnection::new(self.config.base_config.clone(), pool_tx.clone()).await?;

            connection.connect().await?;
            connections.insert(connection.id.clone(), Arc::new(connection));
        }

        let event_broadcast = self.event_broadcast.clone();
        tokio::spawn(async move {
            while let Some(event) = pool_rx.recv().await {
                match event {
                    PoolEvent::ConnectionEvent { event, .. } => {
                        _ = event_broadcast.send(event);
                    }
                    _ => {}
                }
            }
        });

        Ok(())
    }

    async fn start_healthcheck(&self) {
        let connections = self.connections.clone();
        let interval = self.config.health_check_interval;

        tokio::spawn(async move {
            let mut tick = tokio::time::interval(interval);
            loop {
                tick.tick().await;

                let connections = connections.read().await;
                for connection in connections.values() {
                    let stats = connection.get_stats().await;
                    debug!(
                        "healthcheck: connection {}: {} channels, connected: {}",
                        stats.id, stats.channel_count, stats.is_connected
                    );

                    if !stats.is_connected
                        && stats.last_activity.elapsed() > Duration::from_secs(300)
                    {
                        warn!("connection '{}': possibly bad!", stats.id);
                    }
                }
            }
        });
    }

    async fn start_rebalancer(&self) {
        let _connections = self.connections.clone();
        let _channel_map = self.channel_map.clone();
        let interval = self.config.rebalance_interval;

        tokio::spawn(async move {
            let mut tick = tokio::time::interval(interval);

            loop {
                tick.tick().await;
                info!("rebalancing pool...");

                // do some kind of rebalance operation
                // cant be bothered at this moment...
            }
        });
    }

    pub async fn join_channel(&self, channel: &str) -> IrcResult<()> {
        let (tx, rx) = oneshot::channel();
        self.command_tx
            .send(PoolCommand::JoinChannel {
                channel: channel.to_string(),
                response: tx,
            })
            .map_err(|_| IrcError::ConnectionFailed("pool command channel closed".to_string()))?;

        rx.await
            .map_err(|_| IrcError::ConnectionFailed("response channel closed".to_string()))?
    }

    pub async fn leave_channel(&self, channel: &str) -> IrcResult<()> {
        let (tx, rx) = oneshot::channel();
        self.command_tx
            .send(PoolCommand::LeaveChannel {
                channel: channel.to_string(),
                response: tx,
            })
            .map_err(|_| IrcError::ConnectionFailed("pool command channel closed".to_string()))?;

        rx.await
            .map_err(|_| IrcError::ConnectionFailed("response channel closed".to_string()))?
    }

    pub async fn send_message(&self, channel: &str, message: &str) -> IrcResult<()> {
        let (tx, rx) = oneshot::channel();
        self.command_tx
            .send(PoolCommand::SendMessage {
                channel: channel.to_string(),
                message: message.to_string(),
                response: tx,
            })
            .map_err(|_| IrcError::ConnectionFailed("pool command channel closed".to_string()))?;

        rx.await
            .map_err(|_| IrcError::ConnectionFailed("response channel closed".to_string()))?
    }

    pub async fn get_stats(&self) -> PoolStats {
        let (tx, rx) = oneshot::channel();
        if self
            .command_tx
            .send(PoolCommand::GetStats { response: tx })
            .is_ok()
        {
            rx.await.unwrap_or_else(|_| PoolStats {
                total_connections: 0,
                active_connections: 0,
                total_channels: 0,
                average_load: 0.0,
                connections: Vec::new(),
            })
        } else {
            PoolStats {
                total_connections: 0,
                active_connections: 0,
                total_channels: 0,
                average_load: 0.0,
                connections: Vec::new(),
            }
        }
    }

    pub async fn shutdown(&self) {
        let (tx, rx) = oneshot::channel();
        if self
            .command_tx
            .send(PoolCommand::Shutdown { response: tx })
            .is_ok()
        {
            _ = rx.await;
        }
    }
}

pub struct PoolManager {
    config: PoolConfig,
    connections: Arc<RwLock<HashMap<String, Arc<PooledConnection>>>>,
    channel_map: Arc<RwLock<HashMap<String, String>>>,
    event_broadcast: broadcast::Sender<IrcEvent>,
    load_balancing: BalancingStrategy,
    next_connection_index: Arc<AtomicUsize>,
}

impl PoolManager {
    fn new(
        config: PoolConfig,
        connections: Arc<RwLock<HashMap<String, Arc<PooledConnection>>>>,
        channel_map: Arc<RwLock<HashMap<String, String>>>,
        event_broadcast: broadcast::Sender<IrcEvent>,
        load_balancing: BalancingStrategy,
        next_connection_index: Arc<AtomicUsize>,
    ) -> Self {
        Self {
            config,
            connections,
            channel_map,
            event_broadcast,
            load_balancing,
            next_connection_index,
        }
    }

    async fn run(self, mut command_rx: mpsc::UnboundedReceiver<PoolCommand>) {
        info!("running pool manager");

        while let Some(command) = command_rx.recv().await {
            match command {
                PoolCommand::JoinChannel { channel, response } => {
                    let res = self.handle_join_channel(&channel).await;
                    _ = response.send(res);
                }
                PoolCommand::LeaveChannel { channel, response } => {
                    let res = self.handle_leave_channel(&channel).await;
                    _ = response.send(res);
                }
                PoolCommand::SendMessage {
                    channel,
                    message,
                    response,
                } => {
                    let res = self.handle_send_message(&channel, &message).await;
                    _ = response.send(res);
                }
                PoolCommand::GetStats { response } => {
                    let res = self.collect_stats().await;
                    _ = response.send(res);
                }
                PoolCommand::Shutdown { response } => {
                    info!("shutting down pool manager");
                    _ = response.send(());
                    break;
                }
            }
        }

        info!("pool manager stopped");
    }

    async fn handle_join_channel(&self, channel: &str) -> IrcResult<()> {
        if self.channel_map.read().await.contains_key(channel) {
            return Ok(());
        }

        let connection = self.select_connection_for_channel().await?;
        connection.join_channel(channel).await?;

        self.channel_map
            .write()
            .await
            .insert(channel.to_string(), connection.id.clone());

        self.check_scale_up().await?;
        Ok(())
    }

    async fn handle_leave_channel(&self, channel: &str) -> IrcResult<()> {
        let connection_id = {
            let channel_map = self.channel_map.read().await;
            channel_map.get(channel).cloned()
        };

        if let Some(connection_id) = connection_id {
            let connections = self.connections.read().await;
            if let Some(connection) = connections.get(&connection_id) {
                connection.leave_channel(channel).await?;
                drop(connections);

                self.channel_map.write().await.remove(channel);
                self.check_scale_down().await?;
            }
        }

        Ok(())
    }

    async fn handle_send_message(&self, channel: &str, message: &str) -> IrcResult<()> {
        let connection_id = {
            let channel_map = self.channel_map.read().await;
            channel_map.get(channel).cloned()
        };

        if let Some(connection_id) = connection_id {
            let connections = self.connections.read().await;
            if let Some(connection) = connections.get(&connection_id) {
                return connection.send_message(channel, message).await;
            }
        }

        Err(IrcError::ConnectionFailed(format!(
            "no connection for '{}'",
            channel
        )))
    }

    async fn collect_stats(&self) -> PoolStats {
        let connections = self.connections.read().await;
        let mut connection_stats = Vec::new();
        let mut total_channels = 0;
        let mut active_connections = 0;

        for connection in connections.values() {
            let stats = connection.get_stats().await;
            total_channels += stats.channel_count;
            if stats.is_connected {
                active_connections += 1;
            }
            connection_stats.push(stats);
        }

        let total_capacity = connections.len() * self.config.max_per_connection;
        let average_load = if total_capacity > 0 {
            total_channels as f64 / total_capacity as f64
        } else {
            0.0
        };

        PoolStats {
            total_connections: connections.len(),
            active_connections,
            total_channels,
            average_load,
            connections: connection_stats,
        }
    }

    async fn select_connection_for_channel(&self) -> IrcResult<Arc<PooledConnection>> {
        let connections = self.connections.read().await;

        match self.load_balancing {
            BalancingStrategy::LeastLoaded => {
                let mut best = None;
                let mut min_channels = usize::MAX;

                for connection in connections.values() {
                    let channel_count = connection.channel_count().await;
                    if channel_count < self.config.max_per_connection
                        && channel_count < min_channels
                    {
                        min_channels = channel_count;
                        best = Some(connection.clone());
                    }
                }

                best.ok_or_else(|| {
                    IrcError::ConnectionFailed("no available connections".to_string())
                })
            }

            BalancingStrategy::RoundRobin => {
                let connections_vec: Vec<_> = connections.values().cloned().collect();
                if connections_vec.is_empty() {
                    return Err(IrcError::ConnectionFailed(
                        "no available connections".to_string(),
                    ));
                }

                let index = self
                    .next_connection_index
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
                    % connections_vec.len();

                Ok(connections_vec[index].clone())
            }
            BalancingStrategy::Random => {
                let connections_vec: Vec<_> = connections.values().cloned().collect();
                if connections_vec.is_empty() {
                    return Err(IrcError::ConnectionFailed(
                        "no connections available".to_string(),
                    ));
                }

                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};

                let mut hasher = DefaultHasher::new();
                std::time::SystemTime::now().hash(&mut hasher);
                let index = (hasher.finish() as usize) % connections_vec.len();

                Ok(connections_vec[index].clone())
            }
        }
    }

    async fn check_scale_up(&self) -> IrcResult<()> {
        let connections = self.connections.read().await;

        if connections.len() >= self.config.max_connections {
            return Ok(());
        }

        let mut total_channels = 0;
        let mut total_capacity = 0;

        for connection in connections.values() {
            let channel_count = connection.channel_count().await;
            total_channels += channel_count;
            total_capacity += self.config.max_per_connection;
        }

        let current_load = total_channels as f64 / total_capacity as f64;
        if current_load > self.config.scale_up_threshold {
            info!(
                "scaling up: current load {:.2}%; threshold {:.2}",
                current_load * 100.0,
                self.config.scale_up_threshold * 100.0
            );

            drop(connections);
            // self.create_new_connection().await?;
            //
            // no scale down implemented so im going to leave
            // this for now
        }

        Ok(())
    }

    async fn check_scale_down(&self) -> IrcResult<()> {
        let connections = self.connections.read().await;

        if connections.len() <= self.config.min_connections {
            return Ok(());
        }

        info!("scale down check - current: {}", connections.len());
        // scale down
        // idk

        Ok(())
    }

    async fn create_new_connection(&self) -> IrcResult<()> {
        let (pool_tx, _) = mpsc::unbounded_channel();
        let mut connection =
            PooledConnection::new(self.config.base_config.clone(), pool_tx.clone()).await?;

        info!("creating new connection {}...", connection.id);

        connection.connect().await?;

        let mut connections = self.connections.write().await;
        connections.insert(connection.id.clone(), Arc::new(connection));

        info!("created; total: {}", connections.len());
        Ok(())
    }
}

pub async fn refresh_channels() -> SocketPoolResult<HashMap<String, String>> {
    let updated_channels = channel::get_tracked_channels().await?;
    Ok(updated_channels)
}
