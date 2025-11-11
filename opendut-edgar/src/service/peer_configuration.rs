use crate::service::can::can_manager::CanManagerRef;
use crate::service::network_interface::manager::NetworkInterfaceManagerRef;
use crate::service::tasks;
use crate::service::test_execution::executor_manager::ExecutorManagerRef;
use opendut_model::peer::configuration::{EdgePeerConfigurationState, PeerConfiguration};

use std::fmt::Formatter;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, error};
use crate::service;
use crate::service::service_runner::CollectedResult;
use super::network_metrics::manager::NetworkMetricsManagerRef;

#[derive(Debug)]
pub struct ApplyPeerConfigurationParams {
    pub peer_configuration: PeerConfiguration,
    pub network_interface_management: NetworkInterfaceManagement,
    pub executor_manager: ExecutorManagerRef,
    pub metrics_manager: NetworkMetricsManagerRef,
}
#[derive(Clone)]
pub enum NetworkInterfaceManagement {
    Enabled { network_interface_manager: NetworkInterfaceManagerRef, can_manager: CanManagerRef },
    Disabled,
}
impl std::fmt::Debug for NetworkInterfaceManagement {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            NetworkInterfaceManagement::Enabled { .. } => writeln!(f, "Enabled"),
            NetworkInterfaceManagement::Disabled => writeln!(f, "Disabled"),
        }
    }
}

pub async fn spawn_peer_configurations_handler(
    mut rx_peer_configuration: mpsc::Receiver<ApplyPeerConfigurationParams>,
    tx_peer_configuration_state: mpsc::Sender<EdgePeerConfigurationState>
) -> anyhow::Result<()> {
    tokio::spawn(async move {
        while let Some(apply_peer_configuration_params) = rx_peer_configuration.recv().await {
            let result = apply_peer_configuration(apply_peer_configuration_params).await;
            let state = EdgePeerConfigurationState::from(result);
            let _ = tx_peer_configuration_state.send(state).await
                .inspect_err(|err| error!("Failed to send peer configuration state to CARL. {err}"));
        }
    });
    Ok(())
}

#[tracing::instrument(skip_all)]
async fn apply_peer_configuration(params: ApplyPeerConfigurationParams) -> CollectedResult {
    let ApplyPeerConfigurationParams { 
        peer_configuration,
        network_interface_management, 
        executor_manager, metrics_manager } = params;

    let resolver = tasks::task_resolver::ServiceTaskResolver::new(
        peer_configuration.clone(),
        network_interface_management.clone(),
        Arc::clone(&metrics_manager),
    );
    let result = service::service_runner::run_tasks(peer_configuration.clone(), resolver).await;
    if result.success {
        debug!("Peer configuration tasks executed successfully: {:?}", result.items);
    } else {
        error!("Failed to apply peer configuration tasks. Following tasks failed: {:?}", result.items);
        return result;
    }

    {
        let mut executor_manager = executor_manager.lock().await;
        executor_manager.terminate_executors();
        executor_manager.create_new_executors(peer_configuration.executors);
    }

    debug!("Peer configuration has been successfully applied.");
    result
}

