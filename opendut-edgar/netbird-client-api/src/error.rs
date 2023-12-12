use std::backtrace::Backtrace;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{message}: {cause}\n{backtrace}")]
    Transport { message: String, cause: tonic::transport::Error, backtrace: Backtrace },
    #[error("Request error: {cause}\n{backtrace}")]
    Request { #[from] cause: tonic::Status, backtrace: Backtrace }
}
impl Error {
    pub fn transport(cause: tonic::transport::Error, message: impl Into<String>) -> Self {
        Error::Transport {
            message: message.into(),
            cause,
            backtrace: Backtrace::capture(),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
