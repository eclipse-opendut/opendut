use crate::service::process_manager::{AsyncProcessId, AsyncProcessManagerExt, AsyncProcessManagerRef};
use opendut_model::peer::configuration::parameter::CanConnection;
use opendut_model::peer::configuration::{ParameterId, ParameterValue};
use std::collections::HashMap;
use std::process::Stdio;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::process::Command;

pub type CanManagerRef = Arc<Mutex<CanManager>>;

pub trait CanManagerExt {
    fn new_shared() -> Self;
}

impl CanManagerExt for CanManagerRef {
    fn new_shared() -> CanManagerRef {
        let process_manager = AsyncProcessManagerRef::new_shared();
        Arc::new(Mutex::new(CanManager::create(process_manager)))
    }
}

pub struct CanManager {
    process_manager: AsyncProcessManagerRef,
    process_map: HashMap<ParameterId, AsyncProcessId>,
}

impl CanManager {
    fn create(process_manager: AsyncProcessManagerRef) -> Self {
        Self {
            process_manager,
            process_map: Default::default(),
        }
    }

    pub async fn spawn_process(&mut self, parameter: &CanConnection) -> anyhow::Result<()> {
        let id = parameter.parameter_identifier();
        let mut cmd = Command::new("cannelloni");
        Self::fill_cannelloni_cmd(&parameter, &mut cmd);
        let mut process_manager = self.process_manager.lock().await;
        let process_id = process_manager.spawn("foo", &mut cmd).await?;
        self.process_map.insert(id.clone(), process_id);

        Ok(())
    }

    pub async fn process_is_running(&mut self, parameter: &CanConnection) -> anyhow::Result<bool> {
        let id = parameter.parameter_identifier();
        if let Some(process_id) = self.process_map.get(&id) {
            let mut process_manager = self.process_manager.lock().await;
            Ok(process_manager.process_is_running(process_id))
        } else {
            Ok(false)
        }
    }

    pub async fn terminate_process(&mut self, parameter: &CanConnection) -> anyhow::Result<()> {
        let id = parameter.parameter_identifier();
        let mut process_manager = self.process_manager.lock().await;
        if let Some(process_id) = self.process_map.remove(&id) {
            process_manager.terminate(process_id).await?;
        }
        Ok(())
    }

    /*
     cannelloni with SCTP transport for CAN bus tunneling

     With SCTP it is possible to use cannelloni over lossy connections where packet loss and re-ordering is very likely.
     The SCTP implementation uses the server-client model (for now). One instance binds on a fixed port and the other instance (client) connects to it.

     https://github.com/mguentner/cannelloni?tab=readme-ov-file#sctp
     */
    fn fill_cannelloni_cmd(parameter: &CanConnection, cmd: &mut Command) {
        let instance_type = if parameter.local_is_server {"s"} else {"c"}; // act as server or client
        let port_arg = if parameter.local_is_server {"-l"} else {"-r"};  // listening port or remote port
        let port = if parameter.local_is_server {
            parameter.local_port
        } else {
            parameter.remote_port
        };

        cmd.arg("-I")
            .arg(parameter.can_interface_name.name())
            .arg("-S")  // enable SCTP transport
            .arg(instance_type)
            .arg("-t")  // buffer timeout
            .arg(parameter.buffer_timeout_microseconds.to_string())
            .arg("-R")  // remote IP address
            .arg(parameter.remote_ip.to_string())
            .arg(port_arg)
            .arg(port.to_string())
            .stderr(Stdio::piped())
            .stdout(Stdio::piped());
    }

}
