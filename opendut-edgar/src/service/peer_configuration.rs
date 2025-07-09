use crate::service::can::can_manager::CanManagerRef;
use crate::service::can::cluster_assignment;
use crate::service::network_interface::manager::NetworkInterfaceManagerRef;
use crate::service::tasks;
use crate::service::test_execution::executor_manager::ExecutorManagerRef;
use opendut_types::cluster::ClusterAssignment;
use opendut_types::peer::configuration::{parameter, OldPeerConfiguration, ParameterField, PeerConfiguration, PeerConfigurationState};
use opendut_types::peer::PeerId;

use std::fmt::Formatter;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, error};
use crate::common::task;
use crate::common::task::service_runner::CollectedResult;
use super::network_metrics::manager::NetworkMetricsManagerRef;

#[derive(Debug)]
pub struct ApplyPeerConfigurationParams {
    pub self_id: PeerId,
    pub peer_configuration: PeerConfiguration,
    pub old_peer_configuration: OldPeerConfiguration,
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

pub async fn spawn_peer_configurations_handler(mut rx_peer_configuration: mpsc::Receiver<ApplyPeerConfigurationParams>, _tx_peer_configuration_state: mpsc::Sender<PeerConfigurationState>) -> anyhow::Result<()> {
    tokio::spawn(async move {
        while let Some(apply_peer_configuration_params) = rx_peer_configuration.recv().await {
            let _ = apply_peer_configuration(apply_peer_configuration_params).await
                .inspect_err(|error| error!("Error while applying peer configuration: {error}"));
        }
    });
    Ok(())
}

#[tracing::instrument(skip_all)]
async fn apply_peer_configuration(params: ApplyPeerConfigurationParams) -> anyhow::Result<()> {
    let ApplyPeerConfigurationParams { 
        self_id, peer_configuration, old_peer_configuration, 
        network_interface_management, 
        executor_manager, metrics_manager } = params;

    let resolver = tasks::task_resolver::ServiceTaskResolver::new(
        peer_configuration.clone(),
        network_interface_management.clone(),
        Arc::clone(&metrics_manager),
    );
    let CollectedResult { items, success } = task::service_runner::run_tasks(peer_configuration.clone(), resolver).await;
    if success {
        debug!("Peer configuration tasks executed successfully: {items:?}");
    } else {
        return Err(anyhow::anyhow!("Failed to apply peer configuration tasks. Following tasks failed: {items:?}"));
    }
    // TODO: Send feedback of results to CARL

    if let NetworkInterfaceManagement::Enabled { can_manager, .. } = &network_interface_management {
        let _ = setup_can(
            &old_peer_configuration.cluster_assignment,
            peer_configuration.device_interfaces,
            self_id,
            Arc::clone(can_manager),
        ).await;
    } else {
        debug!("Skipping changes to CAN interfaces, since network interface management is disabled.");
    }

    {
        let mut executor_manager = executor_manager.lock().await;
        executor_manager.terminate_executors();
        executor_manager.create_new_executors(peer_configuration.executors);
    }

    debug!("Peer configuration has been successfully applied.");
    Ok(())
}

#[tracing::instrument(skip_all)]
async fn setup_can( //TODO make CAN idempotent
    cluster_assignment: &Option<ClusterAssignment>,
    device_interfaces: ParameterField<parameter::DeviceInterface>,
    self_id: PeerId,
    can_manager: CanManagerRef,
) -> anyhow::Result<()> {

    match cluster_assignment {
        Some(cluster_assignment) => {
            cluster_assignment::setup_can_interfaces(
                cluster_assignment,
                self_id,
                device_interfaces,
                can_manager,
            ).await
            .inspect_err(|error| error!("Failed to configure CAN interfaces: {error}"))?;
        }
        None => {
            debug!("No ClusterAssignment in peer configuration, not setting up CAN.");
            //TODO teardown cluster, if configuration changed
        }
    }
    Ok(())
}
