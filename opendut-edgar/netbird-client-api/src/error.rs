#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{message}: {cause}")]
    Transport { message: String, cause: tonic::transport::Error },
    #[error("Request error: {cause}")]
    Request { #[from] cause: tonic::Status }
}
impl Error {
    pub fn transport(cause: tonic::transport::Error, message: impl Into<String>) -> Self {
        Error::Transport {
            message: message.into(),
            cause,
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
