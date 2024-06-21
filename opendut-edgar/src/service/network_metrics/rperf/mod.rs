use std::io::Error;
use std::num::ParseFloatError;

pub mod client;
pub mod server;

#[derive(thiserror::Error, Debug)]
pub enum RperfError {
    #[error("'{message}'. Cause: '{cause}'")]
    Start { message: String, cause: Error },
    #[error("{message}\n")]
    StdoutAccess { message: String},
    #[error("{message}\n")]
    StderrAccess { message: String},
    #[error("{message}\n  {cause}")]
    BandwidthParse { message: String, cause: ParseFloatError },
    #[error("Client error: '{message}'.")]
    Other { message: String },
}
#[derive(thiserror::Error, Debug)]
pub enum RperfRunError {
    #[error("RperfClientError: '{message}'. Cause: '{cause}'")]
    RperfClientError { message: String, cause: RperfError },
    #[error("RperfServerError: '{message}'. Cause: '{cause}'")]
    RperfServerError { message: String, cause: RperfError },
}
