use log::SetLoggerError;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Unable to set logger: {source}")]
    SetLogger { #[from] source: SetLoggerError },
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
        .parse_env("OPENDUT_LOG");

    let builder = overrides(builder);

    builder
        .try_init()
        .map_err(Error::from)
}
