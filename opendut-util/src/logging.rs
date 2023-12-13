use std::backtrace::Backtrace;
use std::fmt::Display;

use log::SetLoggerError;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Unable to set logger: {}\n{backtrace}", source)]
    SetLogger { #[from] source: SetLoggerError, backtrace: Backtrace },
}

pub fn initialize() -> Result<(), Error> {
    initialize_with_overrides(|builder| builder)
}

pub fn initialize_with_overrides(overrides: fn(&mut env_logger::Builder) -> &mut env_logger::Builder) -> Result<(), Error> {
    let mut builder = env_logger::builder();

    let builder = builder
        .format_timestamp_millis()
        .filter_level(log::LevelFilter::Info)
        .filter_module("opendut", log::LevelFilter::Trace)
        .parse_default_env();

    let builder = overrides(builder);

    builder
        .try_init()
        .map_err(Error::from)
}

pub trait LogError<T, E>
where
    E: Display
{
    fn log_err(self) -> Result<T, E>;
    fn err_logged(self);
}

impl <T, E> LogError<T, E> for Result<T, E>
where
    E: Display
{
    fn log_err(self) -> Result<T, E>
    where
        E: Display
    {
        match self {
            Ok(t) => Ok(t),
            Err(e) => {
                log::error!("{e}");
                Err(e)
            },
        }
    }

    fn err_logged(self) {
        if let Err(err) = self {
            log::error!("{err}");
        }
    }
}
