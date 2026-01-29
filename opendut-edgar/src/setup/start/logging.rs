use std::env;
use std::path::{Path, PathBuf};
use opendut_telemetry::logging::{PipeLogging, PipeLoggingStream};
use opendut_telemetry::opentelemetry_types::Opentelemetry;


pub async fn init(log_file: &Option<PathBuf>) -> anyhow::Result<()> {
    let log_file = log_file.clone().unwrap_or(default_file()?);

    let (pipe_logging, file_logging) =
        if log_file == Path::new("-") { // `--log-file=-` means to write stdout/stderr
            crate::common::banner::print(); //print the banner to the logs to include version information and make it easy to spot the start of the logs; we don't do that into the file, because using log statements would prefix it with a timestamp

            interactive_messages::disable(); //disable interactive messages, because we want to print logs to stdout/stderr instead
            let pipe_logging = PipeLogging::Enabled { stream: PipeLoggingStream::Stderr };
            (pipe_logging, None)
        } else {
            (PipeLogging::Disabled, Some(log_file))
        };

    let logging_config = opendut_telemetry::logging::LoggingConfig {
        pipe_logging,
        file_logging,
        log_level_override: None,
    };
    let opentelemetry_config = Opentelemetry::Disabled;

    let _ = opendut_telemetry::initialize_with_config(logging_config, opentelemetry_config).await?;

    Ok(())
}
fn default_file() -> anyhow::Result<PathBuf> {
    let mut log_file = env::current_exe()?;
    log_file.set_file_name("setup.log");
    Ok(log_file)
}


/// Controls whether `println!()` calls intended for interactive use are shown to users on stdout/stderr.
pub mod interactive_messages {
    use std::sync::atomic::{AtomicBool, Ordering};

    /// Whether the `interactive_message!()` macro prints anything.
    pub static INTERACTIVE_MESSAGES_ENABLED: AtomicBool = AtomicBool::new(true);

    pub fn is_enabled() -> bool {
        INTERACTIVE_MESSAGES_ENABLED.load(Ordering::Relaxed)
    }

    pub fn disable() {
        INTERACTIVE_MESSAGES_ENABLED.store(false, Ordering::Relaxed);
    }

    /// Write a message for an interactive user to see.
    /// Prefer this over `println!()`, since we can disable it in non-interactive use.
    #[macro_export]
    macro_rules! interactive_message {
        ( $($arg:tt)* ) => {
            if $crate::setup::start::logging::interactive_messages::is_enabled() {
                eprintln!($($arg)*);
            }
        };
    }
}
