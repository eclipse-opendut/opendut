use std::fs::File;
use std::path::PathBuf;
use std::str::FromStr;

use tracing_subscriber::filter::Directive;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::filter::{EnvFilter, LevelFilter};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Unable to initialize tracing: {source}")]
    TracingFilterFromEnv { #[from] source: tracing_subscriber::filter::FromEnvError },
    #[error("Unable to initialize tracing: {source}")]
    TracingFilterParse { #[from] source: tracing_subscriber::filter::ParseError },
    #[error("Unable to set initialize tracing: {source}")]
    TracingInit { #[from] source: tracing_subscriber::util::TryInitError },
}

pub fn initialize() -> Result<(), Error> {
    initialize_with_config(LoggingConfig::default())
}

pub fn initialize_with_config(config: LoggingConfig) -> Result<(), Error> {

    let tracing_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .with_env_var("OPENDUT_LOG")
        .from_env()?
        .add_directive(Directive::from_str("opendut=trace")?);

    let logging_layer = tracing_subscriber::fmt::layer()
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .compact();

    let file_logging_layer =
        if let Some(log_file) = config.file_logging {

            let log_file = File::create(&log_file)
                .unwrap_or_else(|cause| panic!("Failed to open log file at '{}': {cause}", log_file.display()));

            Some(tracing_subscriber::fmt::layer()
                .with_writer(log_file))
        } else {
            None
        };

    tracing_subscriber::registry()
        .with(tracing_filter)
        .with(logging_layer)
        .with(file_logging_layer)
        .try_init()?;

    Ok(())
}

#[derive(Default)]
pub struct LoggingConfig {
    pub file_logging: Option<PathBuf>,
}