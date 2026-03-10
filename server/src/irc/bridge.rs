#![allow(dead_code)]

use tokio::sync::{mpsc, oneshot};
use tracing::instrument;

use crate::irc::commands::{IrcQuery, OutgoingCommand};
use crate::irc::connection::ConnectionHandle;
use crate::irc::error::ClientResult;

#[derive(Clone, Debug)]
pub struct IrcHandle {
    pub cmd_tx: mpsc::Sender<OutgoingCommand>,
    pub query_tx: mpsc::Sender<IrcQuery>,
    /// Used to trigger connection resets
    pub connection: ConnectionHandle,
}

impl IrcHandle {
    #[instrument(skip(self))]
    pub async fn joined_channels(&self) -> ClientResult<Vec<String>> {
        let (tx, rx) = oneshot::channel();
        self.query_tx
            .send(IrcQuery::GetJoinedChannels { reply: tx })
            .await?;

        Ok(rx.await?)
    }

    // pub async fn send_reply(
    //     &self,
    //     channel: String,
    //     reply_id: String,
    //     message: String,
    // ) -> ClientResult<()> {
    //     self.cmd_tx
    //         .send(OutgoingCommand::Reply {
    //             channel,
    //             reply_id,
    //             message,
    //         })
    //         .await?;
    //
    //     Ok(())
    // }

    #[instrument]
    pub async fn force_reconnect(&mut self) {
        tracing::warn!("connection reset requested");
        _ = self.connection.reset_tx.send(()).await;
    }
}
