use tokio::sync::mpsc::error::SendError;
use opendut_carl_api::proto::services::peer_messaging_broker::downstream;

use opendut_types::peer::PeerId;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("DownstreamSend Error: {0}")]
    DownstreamSend(SendError<downstream::Message>),
    #[error("PeerNotFound Error: {0}")]
    PeerNotFound(PeerId),
    #[error("Other Error: {message}")]
    Other { message: String },
}
pub type Result<T> = std::result::Result<T, Error>;
