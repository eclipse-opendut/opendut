use std::collections::HashMap;
use tokio::process::{Child, Command};
use tokio::time::Duration;
use tracing::{debug, error, info, warn};

/// A unique identifier for a managed async process
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AsyncProcessId(u64);

impl AsyncProcessId {
    fn new(id: u64) -> Self {
        Self(id)
    }
}

/// Represents a managed external async process
struct ManagedAsyncProcess {
    name: String,
    child: Child,
}

impl ManagedAsyncProcess {
    /// Terminate the process gracefully, then forcefully if needed
    async fn terminate(&mut self) -> anyhow::Result<()> {
        let pid = self.child.id();
        debug!("Terminating async process '{}' (PID: {:?})", self.name, pid);

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
}

impl Drop for ManagedAsyncProcess {
    fn drop(&mut self) {
        self.terminate_blocking();
    }
}

/// Manager for external async OS processes with guaranteed cleanup
pub struct AsyncProcessManager {
    processes: HashMap<AsyncProcessId, ManagedAsyncProcess>,
    next_id: u64,
}

impl AsyncProcessManager {
    pub fn new() -> Self {
        Self {
            processes: HashMap::new(),
            next_id: 0,
        }
    }

    /// Spawn a new external async process
    pub async fn spawn(&mut self, name: impl Into<String>, command: &mut Command) -> anyhow::Result<AsyncProcessId> {
        let name = name.into();
        debug!("Spawning async process '{}'", name);

        let child = command.spawn()?;

        let pid = child.id();
        let process_id = AsyncProcessId::new(self.next_id);
        self.next_id += 1;

        info!("Spawned async process '{}' with PID: {:?} (AsyncProcessId: {:?})", name, pid, process_id);

        self.processes.insert(process_id, ManagedAsyncProcess {
            name,
            child,
        });

        Ok(process_id)
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

        // Call terminate_blocking on all processes
        for (id, _process) in self.processes.iter_mut() {
            debug!("Terminating async process {:?} during manager drop", id);
            // The Drop impl will be called when the process is removed from the HashMap
        }

        self.processes.clear();
        info!("All async processes terminated (blocking)");
    }

    /// Check if a process is still running
    pub fn process_is_running(&mut self, id: AsyncProcessId) -> bool {
        if let Some(process) = self.processes.get_mut(&id) {
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

    /// Get the number of active processes
    pub fn len(&self) -> usize {
        self.processes.len()
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
pub type AsyncProcessManagerRef = std::sync::Arc<tokio::sync::Mutex<AsyncProcessManager>>;

pub trait AsyncProcessManagerExt {
    fn new_shared() -> Self;
}

impl AsyncProcessManagerExt for AsyncProcessManagerRef {
    fn new_shared() -> Self {
        std::sync::Arc::new(tokio::sync::Mutex::new(AsyncProcessManager::new()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::Duration;

    #[tokio::test]
    async fn test_spawn_and_terminate() {
        let mut manager = AsyncProcessManager::new();

        // Spawn a long-running process
        let id = manager.spawn("sleep", Command::new("sleep").arg("10")).await.unwrap();

        assert!(manager.process_is_running(id));
        assert_eq!(manager.len(), 1);

        // Terminate it
        manager.terminate(id).await.unwrap();

        tokio::time::sleep(Duration::from_millis(200)).await;
        assert!(!manager.process_is_running(id));
        assert_eq!(manager.len(), 0);
    }

    #[tokio::test]
    async fn test_shutdown_all() {
        let mut manager = AsyncProcessManager::new();

        // Spawn multiple processes
        let _id1 = manager.spawn("sleep1", Command::new("sleep").arg("10")).await.unwrap();
        let _id2 = manager.spawn("sleep2", Command::new("sleep").arg("10")).await.unwrap();

        assert_eq!(manager.len(), 2);

        manager.shutdown().await;

        assert_eq!(manager.len(), 0);
    }

    #[tokio::test]
    async fn test_async_usage() {
        let manager = AsyncProcessManagerRef::new_shared();

        let id = {
            let mut manager = manager.lock().await;
            manager.spawn("sleep", Command::new("sleep").arg("5")).await.unwrap()
        };

        {
            let mut manager = manager.lock().await;
            assert!(manager.process_is_running(id));
        }

        {
            let mut manager = manager.lock().await;
            manager.terminate(id).await.unwrap();
        }

        tokio::time::sleep(Duration::from_millis(200)).await;

        {
            let mut manager = manager.lock().await;
            assert!(!manager.process_is_running(id));
        }
    }
}
