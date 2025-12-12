mod config;
mod process;

pub use config::{ProcessConfig, OutputConfig, RestartPolicy};
pub use process::AsyncProcessId;

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tokio::time::Duration;
use tracing::{debug, error, info, warn};
use process::ManagedAsyncProcess;

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
    pub async fn spawn_process(
        manager_ref: AsyncProcessManagerRef,
        config: ProcessConfig,
    ) -> anyhow::Result<AsyncProcessId> {
        let name = config.name.clone();
        let restart_policy = config.restart_policy;

        debug!("Spawning async process '{name}' with restart policy {restart_policy:?}");

        let process_id = {
            let mut manager = manager_ref.lock().await;
            let mut command = config.build_command();
            let child = command.spawn()?;
            let pid = child.id();
            let process_id = AsyncProcessId::new(manager.next_id);
            manager.next_id += 1;

            info!("Spawned async process '{name}' with PID: {pid:?} (AsyncProcessId: {process_id:?})");

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
        manager_ref: AsyncProcessManagerRef,
        id: AsyncProcessId,
        config: ProcessConfig,
        mut shutdown_rx: mpsc::Receiver<()>,
    ) {
        let name = config.name.clone();
        let restart_policy = config.restart_policy;
        let restart_delay = config.restart_delay;

        info!("Starting monitor for process '{name}' (ID: {id:?})");

        tokio::spawn(async move {
            const MONITOR_INTERVAL_MS: u64 = 100;

            loop {
                // Check if we should shutdown
                if shutdown_rx.try_recv().is_ok() {
                    debug!("Monitor for process '{name}' shutting down");
                    break;
                }

                // Check process status
                let should_restart = {
                    let mut manager = manager_ref.lock().await;
                    if let Some(process) = manager.processes.get_mut(&id) {
                        match process.child.try_wait() {
                            Ok(Some(status)) => {
                                info!("Process '{name}' exited with status: {status}");

                                // Decide if we should restart
                                match restart_policy {
                                    RestartPolicy::Always => true,
                                    RestartPolicy::OnFailure => !status.success(),
                                    RestartPolicy::Never => false,
                                }
                            }
                            Ok(None) => false, // Still running
                            Err(error) => {
                                error!("Failed to check process '{name}' status: {error}");
                                false
                            }
                        }
                    } else {
                        debug!("Process '{name}' no longer exists in manager.");
                        break;
                    }
                };

                if should_restart {
                    info!("Restarting process '{name}' after delay of {restart_delay:?}");
                    tokio::time::sleep(restart_delay).await;

                    // Restart the process
                    let mut manager = manager_ref.lock().await;
                    if let Some(old_process) = manager.processes.remove(&id)
                    && let Some(config) = old_process.config.clone() {
                        let mut command = config.build_command();
                        match command.spawn() {
                            Ok(child) => {
                                let pid = child.id();
                                info!("Restarted process '{name}' with PID: {pid:?}");

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
                            Err(error) => {
                                error!("Failed to restart process '{name}': {error}. Will retry after delay.");
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

            info!("Monitor for process '{name}' terminated");
        });
    }

    /// Terminate a specific process by its ID
    pub async fn terminate(&mut self, id: AsyncProcessId) -> anyhow::Result<()> {
        if let Some(mut process) = self.processes.remove(&id) {
            info!("Terminating async process '{}'", process.name);
            process.terminate().await
        } else {
            warn!("Attempted to terminate non-existent async process: {id:?}");
            Ok(())
        }
    }

    /// Terminate all managed processes
    pub async fn shutdown(&mut self) {
        info!("Shutting down all {} managed async processes", self.processes.len());

        let process_ids: Vec<_> = self.processes.keys().copied().collect();

        for id in process_ids {
            if let Err(error) = self.terminate(id).await {
                error!("Failed to terminate async process {id:?}: {error}");
            }
        }

        info!("All async processes terminated.");
    }

    /// Synchronous shutdown for Drop implementation
    fn shutdown_blocking(&mut self) {
        info!("Shutting down all {} managed async processes (blocking)...", self.processes.len());
        self.processes.clear();
        info!("All async processes terminated (blocking).");
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
                Err(error) => {
                    error!("Failed to check async process '{}' status: {error}", process.name);
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
    use tokio::process::Command;

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

        let id = AsyncProcessManager::spawn_process(manager.clone(), config).await.unwrap();

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

        let id = AsyncProcessManager::spawn_process(manager.clone(), config).await.unwrap();

        // Wait for the process to exit and restart a few times
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Terminate the process
        {
            let mut mgr = manager.lock().await;
            mgr.terminate(id).await.unwrap();
        }
    }
}
