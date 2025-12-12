use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;
use tokio::process::Command;

/// Configuration for a managed process with restart capability
#[derive(Clone)]
pub struct ProcessConfig {
    pub(super) name: String,
    command_builder: Arc<dyn Fn() -> Command + Send + Sync>,
    pub(super) restart_policy: RestartPolicy,
    pub(super) restart_delay: Duration,
    pub(super) output_config: OutputConfig,
}

impl ProcessConfig {
    pub fn new(name: impl Into<String>, command_builder: impl Fn() -> Command + Send + Sync + 'static) -> Self {
        Self {
            name: name.into(),
            command_builder: Arc::new(command_builder),
            restart_policy: RestartPolicy::Never,
            restart_delay: Duration::from_secs(1),
            output_config: OutputConfig::default(),
        }
    }

    pub fn with_restart_policy(mut self, policy: RestartPolicy) -> Self {
        self.restart_policy = policy;
        self
    }

    pub fn with_restart_delay(mut self, delay: Duration) -> Self {
        self.restart_delay = delay;
        self
    }

    pub fn with_output_config(mut self, config: OutputConfig) -> Self {
        self.output_config = config;
        self
    }

    pub(super) fn build_command(&self) -> Command {
        let mut command = (self.command_builder)();

        // Configure stdout and stderr based on output config
        match self.output_config {
            OutputConfig::Capture => {
                command.stdout(Stdio::piped());
                command.stderr(Stdio::piped());
            }
            OutputConfig::Inherit => {
                command.stdout(Stdio::inherit());
                command.stderr(Stdio::inherit());
            }
            OutputConfig::Discard => {
                command.stdout(Stdio::null());
                command.stderr(Stdio::null());
            }
        }

        command
    }
}

/// Policy for restarting a process when it terminates
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RestartPolicy {
    /// Never restart the process
    Never,
    /// Always restart the process when it terminates
    Always,
    /// Restart only if the process exits with an error (non-zero exit code)
    #[allow(unused)]
    OnFailure,
}

/// Configuration for process output handling
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OutputConfig {
    /// Capture stdout and stderr for logging when the process exits
    #[default]
    Capture,
    /// Inherit stdout and stderr from parent process
    #[allow(unused)]
    Inherit,
    /// Discard all output (redirect to /dev/null)
    #[allow(unused)]
    Discard,
}
