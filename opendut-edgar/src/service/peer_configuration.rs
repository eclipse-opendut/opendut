use opendut_types::cluster::{ClusterAssignment, PeerClusterAssignment};
use opendut_types::util::net::NetworkInterfaceName;
use tracing::{debug, error, info, trace, warn};
use std::sync::Arc;
use opendut_types::peer::configuration::{OldPeerConfiguration, ParameterTarget, PeerConfiguration};
use opendut_util::project;
use opendut_types::peer::PeerId;
use std::time::Duration;
use std::ops::Not;
use crate::common::task::{runner, Task};
use crate::service::{cluster_assignment, network_metrics, tasks};
use crate::service::can_manager::CanManagerRef;
use crate::service::network_interface::manager::NetworkInterfaceManagerRef;
use crate::service::test_execution::executor_manager::ExecutorManagerRef;
use crate::setup::RunMode;

pub trait PeerConfigurationApplier {
    #[allow(async_fn_in_trait)]
    async fn apply_peer_configuration(
        peer_configuration: PeerConfiguration,
        old_peer_configuration: OldPeerConfiguration,
        apply_config_params: &ApplyConfigurationParams,
    ) -> anyhow::Result<()>;
}

pub struct ApplyConfigurationParams {
    pub self_id: PeerId,
    pub network_interface_management: NetworkInterfaceManagement,
    pub executor_manager: ExecutorManagerRef,
    pub cluster_metrics_options: ClusterMetricsOptions,
}
#[derive(Clone)]
pub enum NetworkInterfaceManagement {
    Enabled { network_interface_manager: NetworkInterfaceManagerRef, can_manager: CanManagerRef },
    Disabled,
}
#[derive(Clone)]
pub struct ClusterMetricsOptions {
    pub ping_interval: Duration,
    pub target_bandwidth_kbit_per_second: u64,
    pub rperf_backoff_max_elapsed_time: Duration,
}

pub struct DefaultPeerConfigurationApplier;
impl PeerConfigurationApplier for DefaultPeerConfigurationApplier {
    async fn apply_peer_configuration(peer_configuration: PeerConfiguration, old_peer_configuration: OldPeerConfiguration, apply_config_params: &ApplyConfigurationParams) -> anyhow::Result<()> {
        {
            let mut tasks: Vec<Box<dyn Task>> = vec![];

            if let NetworkInterfaceManagement::Enabled { network_interface_manager, can_manager: _ } = &apply_config_params.network_interface_management {
                for parameter in peer_configuration.ethernet_bridges.iter().cloned() {
                    tasks.push(Box::new(tasks::create_ethernet_bridge::CreateEthernetBridge {
                        parameter,
                        network_interface_manager: Arc::clone(network_interface_manager),
                    }));
                }
            }

            let no_confirm = true;
            runner::run(RunMode::Service, no_confirm, &tasks).await?;
        }

        {
            let maybe_bridge = peer_configuration.ethernet_bridges.iter()
                .find(|bridge| bridge.target == ParameterTarget::Present); //we currently expect only one bridge to be Present (for one cluster)

            match maybe_bridge {
                Some(bridge) => {
                    let _ = setup_cluster(
                        &old_peer_configuration.cluster_assignment,
                        apply_config_params,
                        &bridge.value.name,
                    ).await;
                }
                None => {
                    debug!("PeerConfiguration contained no info for bridge. Not setting up cluster.");
                }
            }
        }

        let mut executor_manager = apply_config_params.executor_manager.lock().unwrap();
        executor_manager.terminate_executors();
        executor_manager.create_new_executors(peer_configuration.executors);

        setup_cluster_metrics(
            &old_peer_configuration.cluster_assignment,
            apply_config_params.self_id,
            apply_config_params.cluster_metrics_options.clone(),
        )?;
        Ok(())
    }
}

#[tracing::instrument(skip_all)]
async fn setup_cluster( //TODO make idempotent
    cluster_assignment: &Option<ClusterAssignment>,
    apply_config_params: &ApplyConfigurationParams,
    bridge_name: &NetworkInterfaceName,
) -> anyhow::Result<()> {

    match cluster_assignment {
        Some(cluster_assignment) => {
            trace!("Received ClusterAssignment: {cluster_assignment:?}");
            info!("Was assigned to cluster <{}>", cluster_assignment.id);

            if let NetworkInterfaceManagement::Enabled { network_interface_manager, can_manager } = &apply_config_params.network_interface_management {
                cluster_assignment::setup_ethernet_gre_interfaces(
                    cluster_assignment,
                    apply_config_params.self_id,
                    bridge_name,
                    Arc::clone(network_interface_manager),
                ).await
                .inspect_err(|error| error!("Failed to configure Ethernet GRE interfaces: {error}"))?;

                cluster_assignment::join_ethernet_interfaces_to_bridge(
                    cluster_assignment,
                    apply_config_params.self_id,
                    bridge_name,
                    Arc::clone(network_interface_manager),
                ).await
                .inspect_err(|error| error!("Failed to join Ethernet interfaces to bridge: {error}"))?;

                cluster_assignment::setup_can_interfaces(
                    cluster_assignment,
                    apply_config_params.self_id,
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
fn setup_cluster_metrics( //TODO make idempotent
    cluster_assignment: &Option<ClusterAssignment>,
    self_id: PeerId,
    cluster_metrics_options: ClusterMetricsOptions,
) -> anyhow::Result<()> {
    debug!("Setting up cluster metrics.");

    match cluster_assignment {
        None => {}
        Some(cluster_assignment) => {
            let local_peer_assignment = cluster_assignment.assignments.iter().find(|assignment| {
                assignment.peer_id == self_id
            }).ok_or(cluster_assignment::Error::LocalPeerAssignmentNotFound { self_id })?;

            let local_ip = local_peer_assignment.vpn_address;

            let peers: Vec<PeerClusterAssignment> = cluster_assignment.assignments.iter()
                .filter(|peer_cluster_assignment | peer_cluster_assignment.vpn_address != local_ip)
                .cloned().collect();

            let ClusterMetricsOptions { ping_interval, target_bandwidth_kbit_per_second, rperf_backoff_max_elapsed_time } = cluster_metrics_options;

            tokio::spawn(async move {
                network_metrics::ping::cluster_ping(peers.clone(), ping_interval).await;

                if project::is_running_in_development().not() {
                    let _ = network_metrics::rperf::server::exponential_backoff_launch_rperf_server(rperf_backoff_max_elapsed_time).await //ignore errors during startup of rperf server, as we do not want to crash EDGAR for this
                        .inspect_err(|cause| error!("Failed to start rperf server:\n  {cause}"));
                    network_metrics::rperf::client::launch_rperf_clients(peers, target_bandwidth_kbit_per_second, rperf_backoff_max_elapsed_time).await;
                }
            });
        }
    }
    Ok(())
}
