#![allow(dead_code)]

use std::collections::HashSet;
use std::time::Duration;

use tokio::sync::mpsc;
use tracing::instrument;

#[derive(Debug)]
pub enum ChannelEvent {
    /// JOIN observed for our user
    Joined(String),

    /// PART observed for our user
    Parted(String),

    /// Connection up
    Connected,

    /// Connection down
    Disconnected,
}

/// Commands to send back to the supervisor to execute on the socket
#[derive(Debug)]
pub enum ChannelAction {
    Join(Vec<String>),
}

#[derive(Debug)]
pub struct ChannelManager {
    expected: HashSet<String>,
    joined: HashSet<String>,
    event_rx: mpsc::Receiver<ChannelEvent>,
    action_tx: mpsc::Sender<ChannelAction>,
    nick: String,
}

impl ChannelManager {
    #[instrument(skip(event_rx, action_tx))]
    pub fn new(
        channels: Vec<String>,
        nick: String,
        event_rx: mpsc::Receiver<ChannelEvent>,
        action_tx: mpsc::Sender<ChannelAction>,
    ) -> Self {
        let expected: HashSet<String> = channels
            .into_iter()
            .map(|ch| {
                if ch.starts_with('#') {
                    ch
                } else {
                    format!("#{ch}")
                }
            })
            .collect();

        Self {
            expected,
            joined: HashSet::new(),
            event_rx,
            action_tx,
            nick,
        }
    }

    #[instrument(skip(self))]
    pub async fn run(mut self) {
        const MIN_CHECK: Duration = Duration::from_secs(5);
        const MAX_CHECK: Duration = Duration::from_secs(480);

        let mut check_interval = Duration::from_secs(0); // start joining channels immediately
        let mut check_timer = Box::pin(tokio::time::sleep(check_interval));

        tracing::info!("starting channel manager");

        loop {
            tokio::select! {
                Some(event) = self.event_rx.recv() => {
                    match event {
                        ChannelEvent::Joined(channel) => {
                            tracing::debug!(%channel, "JOIN recv");

                            self.joined.insert(channel);
                        }

                        ChannelEvent::Parted(channel) => {
                            tracing::debug!(%channel, "PART recv");

                            self.joined.remove(&channel);

                            if self.expected.contains(&channel) {
                                tracing::warn!(%channel, "attempting rejoin due to unexpected PART");
                                _ = self.action_tx
                                    .send(ChannelAction::Join(vec![channel]))
                                    .await;
                            }
                        }

                        ChannelEvent::Connected => {
                            self.joined.clear();
                            let all: Vec<String> = self.expected.iter().cloned().collect();
                            tracing::info!(count = all.len(), "initial JOIN on connect");

                            _ = self.action_tx
                                .send(ChannelAction::Join(all))
                                .await;

                            check_interval = MIN_CHECK;
                            check_timer.set(tokio::time::sleep(check_interval));
                        }

                        ChannelEvent::Disconnected => {

                            tracing::debug!("supervisor initiated disconnect");
                            self.joined.clear();
                        }
                    }
                }

                _ = check_timer.as_mut() => {
                    let missing: Vec<String> = self.expected
                        .difference(&self.joined)
                        .cloned()
                        .collect::<Vec<_>>();

                    if missing.is_empty() {
                        check_interval = (check_interval * 2).min(MAX_CHECK);
                        tracing::debug!(
                            joined = self.joined.len(),
                            next_check = ?check_interval,
                            "all channels joined",
                        );
                    } else {
                        tracing::warn!(
                            missing_count = missing.len(),
                            ?missing,
                            expected = ?self.expected,
                            "attempting to join missing channels"
                        );

                        let truncate = (5).min(missing.len());

                        _ = self.action_tx
                            .send(ChannelAction::Join(missing[..truncate].to_vec()))
                            .await;

                        check_interval = MIN_CHECK;
                    }

                    check_timer.set(tokio::time::sleep(check_interval));
                }
            }
        }
    }

    #[instrument(skip(self))]
    pub fn add_channel(&mut self, channel: String) {
        self.expected.insert(channel);
    }
}
