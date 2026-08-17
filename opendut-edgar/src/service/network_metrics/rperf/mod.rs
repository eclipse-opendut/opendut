use std::io::Error;
use std::num::ParseFloatError;

pub mod client;
pub mod server;

#[derive(thiserror::Error, Debug)]
pub enum RperfError {
    #[error("'{message}'. Cause: '{source}'")]
    Start { message: String, source: Error },
    #[error("{message}\n")]
    StdoutAccess { message: String},
    #[error("{message}\n")]
    StderrAccess { message: String},
    #[error("{message}\n  {source}")]
    BandwidthParse { message: String, source: ParseFloatError },
    #[error("Client error: '{message}'.")]
    Other { message: String },
}
#[derive(thiserror::Error, Debug)]
pub enum RperfRunError {
    #[error("RperfClientError: '{message}'. Cause: '{source}'")]
    RperfClientError { message: String, source: RperfError },
    #[error("RperfServerError: '{message}'. Cause: '{source}'")]
    RperfServerError { message: String, source: RperfError },
}
