use std::collections::HashMap;
use std::sync::Arc;
use tokio::process::{Child, Command};
use tokio::sync::{mpsc, Mutex};
use tokio::time::Duration;
use std::process::Stdio;
use tracing::{debug, trace, error, info, warn};
use tokio::io::BufReader;
use tokio::io::AsyncBufReadExt;
use tokio::task;


/// A unique identifier for a managed async process
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AsyncProcessId(u64);

impl AsyncProcessId {
    fn new(id: u64) -> Self {
        Self(id)
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


/// Configuration for a managed process with restart capability
#[derive(Clone)]
pub struct ProcessConfig {
    name: String,
    command_builder: Arc<dyn Fn() -> Command + Send + Sync>,
    restart_policy: RestartPolicy,
    restart_delay: Duration,
    output_config: OutputConfig,
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

    fn build_command(&self) -> Command {
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

/// Represents a managed external async process
struct ManagedAsyncProcess {
    name: String,
    child: Child,
    config: Option<ProcessConfig>,
    shutdown_tx: Option<mpsc::Sender<()>>,
}

impl ManagedAsyncProcess {
    /// Terminate the process gracefully, then forcefully if needed
    async fn terminate(&mut self) -> anyhow::Result<()> {
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
                debug!("Process '{}' (PID: {:?}) exited with status: {}", self.name, pid, status);
                Ok(())
            }
            Ok(None) => {
                warn!("Process '{}' (PID: {:?}) did not terminate gracefully, forcing kill", self.name, pid);
                self.child.kill().await?;
                self.child.wait().await?;
                Ok(())
            }
            Err(e) => {
                error!("Failed to check status of process '{}' (PID: {:?}): {}", self.name, pid, e);
                Err(e.into())
            }
        }
    }

    /// Synchronous termination for Drop implementation
    fn terminate_blocking(&mut self) {
        let pid = self.child.id();
        debug!("Terminating async process '{}' (PID: {:?}) in blocking mode", self.name, pid);

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
                warn!("Process '{}' (PID: {:?}) did not terminate gracefully, forcing kill", self.name, pid);
                if let Err(e) = self.child.start_kill() {
                    error!("Failed to kill process '{}' during drop: {}", self.name, e);
                }
                // Give it a moment to die
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
            Err(e) => {
                error!("Failed to check status of process '{}' (PID: {:?}): {}", self.name, pid, e);
            }
        }
    }

    fn try_graceful_termination(&mut self) {
        // Try graceful termination with SIGTERM
        if let Some(pid) = self.child.id() {
            use nix::sys::signal::{kill, Signal};
            use nix::unistd::Pid;

            if let Err(e) = kill(Pid::from_raw(pid as i32), Signal::SIGTERM) {
                warn!("Failed to send SIGTERM to process '{}' (PID: {}): {}", self.name, pid, e);
            }
        }
    }

    /// Drain stdout and stderr asynchronously to avoid deadlock when buffer is full
    fn spawn_output_drainers(&mut self) {
        if let Some(config) = &self.config && let OutputConfig::Capture = config.output_config {
            if let Some(stdout) = self.child.stdout.take() {
                let name = self.name.clone();
                task::spawn(async move {
                    let reader = BufReader::new(stdout);
                    let mut lines = reader.lines();
                    while let Ok(Some(line)) = lines.next_line().await {
                        trace!("{} stdout: {}", name, line);
                    }
                });
            }
            if let Some(stderr) = self.child.stderr.take() {
                let name = self.name.clone();
                task::spawn(async move {
                    let reader = BufReader::new(stderr);
                    let mut lines = reader.lines();
                    while let Ok(Some(line)) = lines.next_line().await {
                        error!("{} stderr: {}", name, line);
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

/// Manager for external async OS processes with guaranteed cleanup and restart capability
pub struct AsyncProcessManager {
    processes: HashMap<AsyncProcessId, ManagedAsyncProcess>,
    next_id: u64,
}

impl AsyncProcessManager {
    fn new() -> Self {
        Self {
            processes: HashMap::new(),
            next_id: 0,
        }
    }

    /// Spawn a process with automatic restart capability
    pub async fn spawn(
        manager_ref: Arc<Mutex<Self>>,
        config: ProcessConfig,
    ) -> anyhow::Result<AsyncProcessId> {
        let name = config.name.clone();
        let restart_policy = config.restart_policy;

        debug!("Spawning async process '{}' with restart policy {:?}", name, restart_policy);

        let process_id = {
            let mut manager = manager_ref.lock().await;
            let mut command = config.build_command();
            let child = command.spawn()?;
            let pid = child.id();
            let process_id = AsyncProcessId::new(manager.next_id);
            manager.next_id += 1;

            info!("Spawned async process '{}' with PID: {:?} (AsyncProcessId: {:?})", name, pid, process_id);

            let (shutdown_tx, shutdown_rx) = mpsc::channel(1);

            let mut managed_process = ManagedAsyncProcess {
                name: name.clone(),
                child,
                config: Some(config.clone()),
                shutdown_tx: Some(shutdown_tx),
            };
            managed_process.spawn_output_drainers();
            manager.processes.insert(process_id, managed_process);

            // Start monitoring if restart policy is set
            if restart_policy != RestartPolicy::Never {
                Self::start_monitoring(
                    manager_ref.clone(),
                    process_id,
                    config,
                    shutdown_rx,
                );
            }

            process_id
        };

        Ok(process_id)
    }

    /// Start monitoring a process for automatic restart
    fn start_monitoring(
        manager_ref: Arc<Mutex<Self>>,
        id: AsyncProcessId,
        config: ProcessConfig,
        mut shutdown_rx: mpsc::Receiver<()>,
    ) {
        let name = config.name.clone();
        let restart_policy = config.restart_policy;
        let restart_delay = config.restart_delay;

        info!("Starting monitor for process '{}' (ID: {:?})", name, id);

        tokio::spawn(async move {
            const MONITOR_INTERVAL_MS: u64 = 100;

            loop {
                // Check if we should shutdown
                if shutdown_rx.try_recv().is_ok() {
                    debug!("Monitor for process '{}' shutting down", name);
                    break;
                }

                // Check process status
                let should_restart = {
                    let mut manager = manager_ref.lock().await;
                    if let Some(process) = manager.processes.get_mut(&id) {
                        match process.child.try_wait() {
                            Ok(Some(status)) => {
                                info!("Process '{}' exited with status: {}", name, status);

                                // Decide if we should restart
                                match restart_policy {
                                    RestartPolicy::Always => true,
                                    RestartPolicy::OnFailure => !status.success(),
                                    RestartPolicy::Never => false,
                                }
                            }
                            Ok(None) => false, // Still running
                            Err(e) => {
                                error!("Failed to check process '{}' status: {}", name, e);
                                false
                            }
                        }
                    } else {
                        debug!("Process '{}' no longer exists in manager", name);
                        break;
                    }
                };

                if should_restart {
                    info!("Restarting process '{}' after delay of {:?}", name, restart_delay);
                    tokio::time::sleep(restart_delay).await;

                    // Restart the process
                    let mut manager = manager_ref.lock().await;
                    if let Some(old_process) = manager.processes.remove(&id)
                        && let Some(config) = old_process.config.clone() {
                        let mut command = config.build_command();
                        match command.spawn() {
                            Ok(child) => {
                                let pid = child.id();
                                info!("Restarted process '{}' with PID: {:?}", name, pid);

                                let (new_shutdown_tx, new_shutdown_rx) = mpsc::channel(1);

                                manager.processes.insert(id, ManagedAsyncProcess {
                                    name: name.clone(),
                                    child,
                                    config: Some(config.clone()),
                                    shutdown_tx: Some(new_shutdown_tx),
                                });

                                // Continue monitoring with new shutdown channel
                                drop(manager); // Release lock
                                shutdown_rx = new_shutdown_rx;
                            }
                            Err(e) => {
                                error!("Failed to restart process '{}': {}. Will retry after delay.", name, e);
                                drop(manager); // Release lock
                                tokio::time::sleep(restart_delay).await;
                            }
                        }
                    }
                } else {
                    // Wait before checking again
                    tokio::time::sleep(Duration::from_millis(MONITOR_INTERVAL_MS)).await;
                }
            }

            info!("Monitor for process '{}' terminated", name);
        });
    }

    /// Terminate a specific process by its ID
    pub async fn terminate(&mut self, id: AsyncProcessId) -> anyhow::Result<()> {
        if let Some(mut process) = self.processes.remove(&id) {
            info!("Terminating async process '{}'", process.name);
            process.terminate().await
        } else {
            warn!("Attempted to terminate non-existent async process: {:?}", id);
            Ok(())
        }
    }

    /// Terminate all managed processes
    pub async fn shutdown(&mut self) {
        info!("Shutting down all {} managed async processes", self.processes.len());

        let process_ids: Vec<_> = self.processes.keys().copied().collect();

        for id in process_ids {
            if let Err(e) = self.terminate(id).await {
                error!("Failed to terminate async process {:?}: {}", id, e);
            }
        }

        info!("All async processes terminated");
    }

    /// Synchronous shutdown for Drop implementation
    fn shutdown_blocking(&mut self) {
        info!("Shutting down all {} managed async processes (blocking)", self.processes.len());
        self.processes.clear();
        info!("All async processes terminated (blocking)");
    }

    /// Check if a process is still running
    pub fn process_is_running(&mut self, id: &AsyncProcessId) -> bool {
        if let Some(process) = self.processes.get_mut(id) {
            match process.child.try_wait() {
                Ok(Some(_)) => {
                    debug!("Async process '{}' has exited", process.name);
                    false
                }
                Ok(None) => true,
                Err(e) => {
                    error!("Failed to check async process '{}' status: {}", process.name, e);
                    false
                }
            }
        } else {
            false
        }
    }

    /// Check if there are no active processes
    pub fn is_empty(&self) -> bool {
        self.processes.is_empty()
    }
}

impl Drop for AsyncProcessManager {
    fn drop(&mut self) {
        self.shutdown_blocking();
    }
}

/// Thread-safe version of AsyncProcessManager
pub type AsyncProcessManagerRef = Arc<Mutex<AsyncProcessManager>>;

pub trait AsyncProcessManagerExt {
    fn new_shared() -> Self;
}

impl AsyncProcessManagerExt for AsyncProcessManagerRef {
    fn new_shared() -> Self {
        Arc::new(Mutex::new(AsyncProcessManager::new()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_restart_on_failure() {
        let manager = AsyncProcessManagerRef::new_shared();

        // Create a process that exits immediately with failure
        let config = ProcessConfig::new(
            "failing-process",
            move || {
                let cmd = Command::new("sh");
                Command::new("sh").arg("-c").arg("exit 1");
                cmd
            }
        )
        .with_restart_policy(RestartPolicy::OnFailure)
        .with_restart_delay(Duration::from_millis(100));

        let id = AsyncProcessManager::spawn(manager.clone(), config).await.unwrap();

        // Wait for the process to fail and restart a few times
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Terminate the process
        {
            let mut mgr = manager.lock().await;
            mgr.terminate(id).await.unwrap();
        }
    }

    #[test_log::test(tokio::test)]
    async fn test_restart_always() {
        let manager = AsyncProcessManagerRef::new_shared();

        // Create a process that exits immediately with success
        let config = ProcessConfig::new(
            "short-lived-process",
            move || Command::new("true")
        )
        .with_restart_policy(RestartPolicy::Always)
        .with_restart_delay(Duration::from_millis(100));

        let id = AsyncProcessManager::spawn(manager.clone(), config).await.unwrap();

        // Wait for the process to exit and restart a few times
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Terminate the process
        {
            let mut mgr = manager.lock().await;
            mgr.terminate(id).await.unwrap();
        }
    }
}
