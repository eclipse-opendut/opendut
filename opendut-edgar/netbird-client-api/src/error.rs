#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{message}: {source}")]
    Transport { message: String, source: tonic::transport::Error },
    #[error("Request error: {source}")]
    Request { #[from] source: tonic::Status }
}
impl Error {
    pub fn transport(source: tonic::transport::Error, message: impl Into<String>) -> Self {
        Error::Transport {
            message: message.into(),
            source,
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
