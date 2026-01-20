use std::net::IpAddr;
use std::time::Duration;
use anyhow::anyhow;
use tokio::process::Command;
use tracing::debug;
use opendut_netbird_client_api::extension::LocalPeerStateExtension;
use crate::service::process_manager::{AsyncProcessId, AsyncProcessManager, AsyncProcessManagerExt, AsyncProcessManagerRef, OutputConfig, ProcessConfig, RestartPolicy};


pub struct NetbirdProcess {
    process_manager: AsyncProcessManagerRef,
    process_id: AsyncProcessId,
}

impl NetbirdProcess {
    pub async fn spawn() -> anyhow::Result<Self> {
        let process_manager = AsyncProcessManagerRef::new_shared();

        let name = "netbird-service";

        let config = ProcessConfig::new(
            name,
            move || {
                let netbird_executable = crate::setup::constants::netbird::unpacked_executable()
                    .expect("Unpacked NetBird executable path should be determinable.");

                let mut command = Command::new(netbird_executable);
                command.arg("service")
                    .arg("run")
                    .arg("--config=/etc/netbird/config.json")
                    .arg("--log-level=info")
                    .arg("--daemon-addr=unix:///var/run/netbird.sock")
                    .arg("--log-file=console");

                command
            }
        )
        .with_restart_policy(RestartPolicy::Always)
        .with_restart_delay(Duration::from_secs(5))
        .with_output_config(OutputConfig::Capture);

        let process_id = AsyncProcessManager::spawn_process(process_manager.clone(), config).await?;

        Ok(Self {
            process_manager,
            process_id,
        })
    }

    pub async fn retrieve_remote_host(&self) -> anyhow::Result<IpAddr> {
        debug!("Determining remote IP address of host in NetBird VPN network.");
        let mut client = opendut_netbird_client_api::client::Client::connect().await?;

        let status = client.full_status().await?;

        debug!("Netbird local peer state {:?}", status.local_peer_state);
        debug!("Netbird management state {:?}", status.management_state);
        debug!("Netbird signal state {:?}", status.signal_state);

        let host = status.local_peer_state
            .ok_or(anyhow!("NetBird Client did not return a local peer state. May not be logged in. Re-run `edgar setup` to fix this."))?
            .local_ip()?;

        Ok(IpAddr::from(host))
    }

    pub async fn terminate(self) -> anyhow::Result<()> {
        self.process_manager.lock().await
            .terminate(self.process_id).await
    }
}
