use crate::service::process_manager::{create_process_log_function, AsyncProcessId, AsyncProcessManager, AsyncProcessManagerExt, AsyncProcessManagerRef, OutputConfig, ProcessConfig, RestartPolicy};
use opendut_model::peer::configuration::parameter::CanConnection;
use opendut_model::peer::configuration::{ParameterId, ParameterValue};
use std::collections::HashMap;
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;
use tokio::process::Command;
use tokio::sync::Mutex;

pub type CanManagerRef = Arc<Mutex<CanManager>>;


pub struct CanManager {
    process_manager: AsyncProcessManagerRef,
    process_map: Mutex<HashMap<ParameterId, AsyncProcessId>>,
}

impl CanManager {
    pub fn create() -> CanManagerRef {
        let process_manager = AsyncProcessManagerRef::new_shared();

        Arc::new(Mutex::new(Self {
            process_manager,
            process_map: Default::default(),
        }))
    }

    pub async fn spawn_process(&self, parameter: &CanConnection) -> anyhow::Result<()> {
        let name = if parameter.local_is_server {
            format!("cannelloni-server-on-port-{}", parameter.port)
        } else {
            format!("cannelloni-to-leader-peer-{}", parameter.remote_peer_id)
        };
        // Create process with restart policy
        let command_parameter = parameter.clone();
        let config = ProcessConfig::new(
            name,
            move || {
                let mut cmd = Command::new("cannelloni");
                Self::fill_cannelloni_cmd(&command_parameter, &mut cmd);
                cmd
            }
        )
            .with_restart_policy(RestartPolicy::Always)
            .with_restart_delay(Duration::from_secs(5))
            .with_output_config(OutputConfig::Capture);

        let process_id = if parameter.local_is_server {
            let local_port = parameter.port.0;
            let log_function = create_process_log_function!("opendut-cannelloni-server", local_port=local_port);
            AsyncProcessManager::spawn_process(self.process_manager.clone(), config, log_function).await?
        } else {
            let remote_peer_id = parameter.remote_peer_id.to_string();
            let log_function = create_process_log_function!("opendut-cannelloni-peer", leader_peer_id=remote_peer_id);
            AsyncProcessManager::spawn_process(self.process_manager.clone(), config, log_function).await?
        };

        let parameter_id = parameter.parameter_identifier();
        let mut processes = self.process_map.lock().await;
        processes.insert(parameter_id, process_id);

        Ok(())
    }

    pub async fn process_is_running(&self, parameter: &CanConnection) -> bool {
        let id = parameter.parameter_identifier();
        let processes = self.process_map.lock().await;
        if let Some(process_id) = processes.get(&id) {
            let mut process_manager = self.process_manager.lock().await;
            process_manager.process_is_running(process_id)
        } else {
            false
        }
    }

    pub async fn terminate_process(&self, parameter: &CanConnection) -> anyhow::Result<()> {
        let id = parameter.parameter_identifier();
        let mut processes = self.process_map.lock().await;
        if let Some(process_id) = processes.remove(&id) {
            let mut process_manager = self.process_manager.lock().await;
            process_manager.terminate(process_id).await?;
        }
        Ok(())
    }

    pub async fn shutdown(&self) {
        let mut process_manager = self.process_manager.lock().await;
        if !process_manager.is_empty() {
            process_manager.shutdown().await;
        }
    }

    /// cannelloni with SCTP transport for CAN bus tunneling
    ///
    /// With SCTP it is possible to use cannelloni over lossy connections where packet loss and re-ordering is very likely.
    /// The SCTP implementation uses the server-client model (for now). One instance binds on a fixed port and the other instance (client) connects to it.
    /// https://github.com/mguentner/cannelloni?tab=readme-ov-file#sctp
    ///
    /// Cannelloni is expected to be replaced with open1722, see https://github.com/eclipse-opendut/opendut/issues/306
    fn fill_cannelloni_cmd(parameter: &CanConnection, cmd: &mut Command) {
        let instance_type = if parameter.local_is_server { "s" } else { "c" }; // act as server or client
        let port_arg = if parameter.local_is_server { "-l" } else { "-r" }; // listening port or remote port

        cmd.arg("-I")
            .arg(parameter.can_interface_name.name())
            .arg("-S")  // enable SCTP transport
            .arg(instance_type)
            .arg("-t")  // buffer timeout
            .arg(parameter.buffer_timeout_microseconds.to_string())
            .arg("-R")  // remote IP address
            .arg(parameter.remote_ip.to_string())
            .arg(port_arg)
            .arg(parameter.port.to_string())
            .stderr(Stdio::piped())
            .stdout(Stdio::piped());
    }
}
