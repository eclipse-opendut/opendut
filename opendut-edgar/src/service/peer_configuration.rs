use std::collections::HashMap;
use std::fmt::Formatter;
use std::net::IpAddr;
use opendut_types::cluster::{ClusterAssignment, PeerClusterAssignment};
use opendut_types::util::net::NetworkInterfaceName;
use tracing::{debug, error, info, trace};
use std::sync::Arc;
use opendut_types::peer::configuration::{parameter, OldPeerConfiguration, Parameter, ParameterTarget, PeerConfiguration};
use opendut_types::peer::PeerId;
use tokio::sync::mpsc;
use crate::common::task::{runner, Task};
use crate::service::{cluster_assignment, tasks};
use crate::service::can_manager::CanManagerRef;
use crate::service::network_interface::manager::NetworkInterfaceManagerRef;
use crate::service::test_execution::executor_manager::ExecutorManagerRef;
use crate::setup::RunMode;

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

pub async fn spawn_peer_configurations_handler(mut rx_peer_configuration: mpsc::Receiver<ApplyPeerConfigurationParams>) -> anyhow::Result<()> {
    tokio::spawn(async move {
        while let Some(apply_peer_configuration_params) = rx_peer_configuration.recv().await {
            apply_peer_configuration(apply_peer_configuration_params).await
                .expect("Error while applying peer configuration.");
        }
    });
    Ok(())
}

#[tracing::instrument(skip_all)]
async fn apply_peer_configuration(params: ApplyPeerConfigurationParams) -> anyhow::Result<()> {
    let ApplyPeerConfigurationParams { self_id, peer_configuration, old_peer_configuration, network_interface_management, executor_manager, metrics_manager } = params;

    {
        let mut tasks: Vec<Box<dyn Task>> = vec![];

        if let NetworkInterfaceManagement::Enabled { network_interface_manager, can_manager: _ } = &network_interface_management {
            for parameter in peer_configuration.ethernet_bridges.iter().cloned() {
                tasks.push(Box::new(tasks::create_ethernet_bridge::CreateEthernetBridge {
                    parameter,
                    network_interface_manager: Arc::clone(network_interface_manager),
                }));
            }
        }

        runner::run(RunMode::Service, &tasks).await?;
    }

    {
        let maybe_bridge = peer_configuration.ethernet_bridges.iter()
            .find(|bridge| bridge.target == ParameterTarget::Present); //we currently expect only one bridge to be Present (for one cluster)

        match maybe_bridge {
            Some(bridge) => {
                let _ = setup_cluster(
                    &old_peer_configuration.cluster_assignment,
                    peer_configuration.device_interfaces,
                    self_id,
                    network_interface_management,
                    &bridge.value.name,
                ).await;
            }
            None => {
                debug!("PeerConfiguration contained no info for bridge. Not setting up cluster.");
            }
        }
    }

    let mut executor_manager = executor_manager.lock().await;
    executor_manager.terminate_executors();
    executor_manager.create_new_executors(peer_configuration.executors);


    if let Some(cluster_assignment) = old_peer_configuration.cluster_assignment {
        setup_cluster_metrics(
            &cluster_assignment.assignments,
            self_id,
            metrics_manager,
        ).await?;
    }

    debug!("Peer configuration has been successfully applied.");
    Ok(())
}

#[tracing::instrument(skip_all)]
async fn setup_cluster( //TODO make idempotent
    cluster_assignment: &Option<ClusterAssignment>,
    device_interfaces: Vec<Parameter<parameter::DeviceInterface>>,
    self_id: PeerId,
    network_interface_management: NetworkInterfaceManagement,
    bridge_name: &NetworkInterfaceName,
) -> anyhow::Result<()> {

    match cluster_assignment {
        Some(cluster_assignment) => {
            trace!("Received ClusterAssignment: {cluster_assignment:?}");
            info!("Was assigned to cluster <{}>", cluster_assignment.id);

            if let NetworkInterfaceManagement::Enabled { network_interface_manager, can_manager } = &network_interface_management {
                cluster_assignment::setup_ethernet_gre_interfaces(
                    cluster_assignment,
                    self_id,
                    bridge_name,
                    Arc::clone(network_interface_manager),
                ).await
                .inspect_err(|error| error!("Failed to configure Ethernet GRE interfaces: {error}"))?;

                cluster_assignment::join_ethernet_interfaces_to_bridge(
                    &device_interfaces,
                    bridge_name,
                    Arc::clone(network_interface_manager),
                ).await
                .inspect_err(|error| error!("Failed to join Ethernet interfaces to bridge: {error}"))?;

                cluster_assignment::setup_can_interfaces(
                    cluster_assignment,
                    self_id,
                    &device_interfaces,
                    Arc::clone(can_manager),
                ).await
                .inspect_err(|error| error!("Failed to configure CAN interfaces: {error}"))?;
            } else {
                debug!("Skipping changes to network interfaces after receiving ClusterAssignment, as this is disabled via configuration.");
            }
        }
        None => {
            debug!("No ClusterAssignment in peer configuration.");
            //TODO teardown cluster, if configuration changed
        }
    }
    Ok(())
}

#[tracing::instrument(skip_all)]
async fn setup_cluster_metrics( //TODO make idempotent
    peer_cluster_assignments: &[PeerClusterAssignment],
    self_id: PeerId,
    metrics_manager: NetworkMetricsManagerRef,
) -> anyhow::Result<()> {

    debug!("Setting up cluster metrics.");

    let remote_peers: HashMap<PeerId, IpAddr> = {
        let local_peer_assignment = peer_cluster_assignments.iter()
            .find(|assignment| assignment.peer_id == self_id)
            .ok_or(cluster_assignment::Error::LocalPeerAssignmentNotFound { self_id })?;

        let local_ip = local_peer_assignment.vpn_address;

        peer_cluster_assignments.iter()
            .filter(|assignment| assignment.vpn_address != local_ip)
            .map(|assignment| (assignment.peer_id, assignment.vpn_address))
            .collect()
    };

    metrics_manager.lock().await
        .set_remote_peers(remote_peers).await;

    Ok(())
}
