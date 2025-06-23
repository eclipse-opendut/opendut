use std::collections::HashMap;
use std::fmt::Formatter;
use opendut_types::cluster::ClusterAssignment;
use tracing::{debug, error};
use std::sync::Arc;
use opendut_types::peer::configuration::{parameter, OldPeerConfiguration, ParameterField, PeerConfiguration};
use opendut_types::peer::PeerId;
use tokio::sync::mpsc;
use crate::common::task::{runner, Task};
use crate::service::tasks;
use crate::service::can::cluster_assignment;
use crate::service::can::can_manager::CanManagerRef;
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

        if let NetworkInterfaceManagement::Enabled { network_interface_manager, .. } = &network_interface_management {
            let mut ethernet_parameters = peer_configuration.ethernet_bridges.clone().into_iter().collect::<Vec<_>>();
            ethernet_parameters.sort_by(|a, b| a.target.cmp(&b.target));
            for parameter in ethernet_parameters {
                tasks.push(Box::new(tasks::create_ethernet_bridge::CreateEthernetBridge {
                    parameter,
                    network_interface_manager: Arc::clone(network_interface_manager),
                }));
            }

            let mut gre_parameters = peer_configuration.gre_interfaces.clone().into_iter().collect::<Vec<_>>();
            gre_parameters.sort_by(|a, b| a.target.cmp(&b.target));
            for parameter in gre_parameters {
                tasks.push(Box::new(tasks::create_gre_interfaces::ManageGreInterface {
                    parameter,
                    network_interface_manager: Arc::clone(network_interface_manager),
                }));
            }

            let mut joined_interfaces = peer_configuration.joined_interfaces.clone().into_iter().collect::<Vec<_>>();
            joined_interfaces.sort_by(|a, b| a.target.cmp(&b.target));
            for parameter in joined_interfaces {
                tasks.push(Box::new(tasks::manage_joined_interfaces::ManageJoinedInterface {
                    parameter,
                    network_interface_manager: Arc::clone(network_interface_manager),
                }));
            }

            {
                let mut remote_peer_connection_checks = peer_configuration.remote_peer_connection_checks.clone().into_iter().collect::<Vec<_>>();
                remote_peer_connection_checks.sort_by(|a, b| a.target.cmp(&b.target));

                let remote_peers = remote_peer_connection_checks.into_iter()
                    .map(|connection_check| (connection_check.value.remote_peer_id, connection_check.value.remote_ip))
                    .collect::<HashMap<_, _>>(); //TODO split into multiple tasks

                tasks.push(Box::new(tasks::setup_cluster_metrics::SetupClusterMetrics {
                    remote_peers,
                    metrics_manager,
                }));
            }
        } else {
            debug!("Skipping changes to Ethernet interfaces, since network interface management is disabled.");
        }

        runner::run(RunMode::Service, &tasks).await?;
    }

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
