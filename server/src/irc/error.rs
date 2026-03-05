use irc::proto::error::MessageParseError;
use thiserror::Error;
use tokio::sync::{AcquireError, mpsc::error::SendError, oneshot::error::RecvError};

use crate::irc::commands::{IrcQuery, OutgoingCommand};

pub type ClientResult<T> = core::result::Result<T, ConnectionClientError>;

#[derive(Debug, Error)]
pub enum ConnectionClientError {
    #[error(transparent)]
    MpscSendOutgoingCommand(#[from] SendError<OutgoingCommand>),

    #[error(transparent)]
    MpscSendIrcQuery(#[from] SendError<IrcQuery>),

    #[error(transparent)]
    MpscRecv(#[from] RecvError),

    #[error(transparent)]
    AcquirePermit(#[from] AcquireError),

    #[error(transparent)]
    MessageParse(#[from] MessageParseError),

    #[error(transparent)]
    EnvError(#[from] crate::util::env::EnvErr),

    #[error(transparent)]
    ClientError(#[from] irc::error::Error),

    #[error(transparent)]
    ChannelError(#[from] crate::util::channel::ChannelError),

    #[error(transparent)]
    PgErr(#[from] crate::db::PgError),

    #[error(transparent)]
    SqlxError(#[from] sqlx::error::Error),

    #[error(transparent)]
    HelixError(#[from] crate::util::helix::HelixErr),
}
