use std::time::Duration;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Child;
use tokio::sync::mpsc;
use tokio::task;
use tracing::{debug, error, trace, warn};
use crate::service::process_manager::{OutputConfig, ProcessConfig};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AsyncProcessId(u64);

impl AsyncProcessId {
    pub(super) fn new(id: u64) -> Self {
        Self(id)
    }
}


pub(super) struct ManagedAsyncProcess {
    pub name: String,
    pub child: Child,
    pub config: Option<ProcessConfig>,
    pub shutdown_tx: Option<mpsc::Sender<()>>,
}

impl ManagedAsyncProcess {
    /// Terminate the process gracefully, then forcefully if needed
    pub async fn terminate(&mut self) -> anyhow::Result<()> {
        let pid = self.child.id();
        debug!("Terminating async process '{}' (PID: {:?})", self.name, pid);

        // Signal the monitor to stop
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(()).await;
        }

        self.try_graceful_termination();

        // Wait a bit for graceful shutdown
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Force kill if still running
        match self.child.try_wait() {
            Ok(Some(status)) => {
                debug!("Process '{}' (PID: {pid:?}) exited with status: {status}", self.name);
                Ok(())
            }
            Ok(None) => {
                warn!("Process '{}' (PID: {pid:?}) did not terminate gracefully, forcing kill", self.name);
                self.child.kill().await?;
                self.child.wait().await?;
                Ok(())
            }
            Err(error) => {
                error!("Failed to check status of process '{}' (PID: {pid:?}): {error}", self.name);
                Err(error.into())
            }
        }
    }

    /// Synchronous termination for Drop implementation
    fn terminate_blocking(&mut self) {
        let pid = self.child.id();
        debug!("Terminating async process '{}' (PID: {pid:?}) in blocking mode", self.name);

        // Drop the shutdown channel
        self.shutdown_tx.take();

        self.try_graceful_termination();

        // Wait a bit for graceful shutdown
        std::thread::sleep(std::time::Duration::from_millis(100));

        // Force kill if still running
        match self.child.try_wait() {
            Ok(Some(status)) => {
                debug!("Process '{}' (PID: {:?}) exited with status: {}", self.name, pid, status);
            }
            Ok(None) => {
                warn!("Process '{}' (PID: {pid:?}) did not terminate gracefully, forcing kill.", self.name);
                if let Err(error) = self.child.start_kill() {
                    error!("Failed to kill process '{}' during drop: {error}", self.name);
                }
                // Give it a moment to die
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
            Err(e) => {
                error!("Failed to check status of process '{}' (PID: {pid:?}): {e}", self.name);
            }
        }
    }

    fn try_graceful_termination(&mut self) {
        // Try graceful termination with SIGTERM
        if let Some(pid) = self.child.id() {
            use nix::sys::signal::{kill, Signal};
            use nix::unistd::Pid;

            if let Err(e) = kill(Pid::from_raw(pid as i32), Signal::SIGTERM) {
                warn!("Failed to send SIGTERM to process '{}' (PID: {pid}): {e}", self.name);
            }
        }
    }

    /// Drain stdout and stderr asynchronously to avoid deadlock when buffer is full
    pub fn spawn_output_drainers(&mut self) {
        if let Some(config) = &self.config && let OutputConfig::Capture = config.output_config {
            if let Some(stdout) = self.child.stdout.take() {
                let name = self.name.clone();
                task::spawn(async move {
                    let reader = BufReader::new(stdout);
                    let mut lines = reader.lines();
                    while let Ok(Some(line)) = lines.next_line().await {
                        trace!("{name} stdout: {line}");
                    }
                });
            }
            if let Some(stderr) = self.child.stderr.take() {
                let name = self.name.clone();
                task::spawn(async move {
                    let reader = BufReader::new(stderr);
                    let mut lines = reader.lines();
                    while let Ok(Some(line)) = lines.next_line().await {
                        error!("{name} stderr: {line}");
                    }
                });
            }
        }
    }
}

impl Drop for ManagedAsyncProcess {
    fn drop(&mut self) {
        self.terminate_blocking();
    }
}
