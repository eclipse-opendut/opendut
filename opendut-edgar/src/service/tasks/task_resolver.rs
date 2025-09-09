use std::collections::HashMap;
use opendut_model::peer::configuration::{ParameterVariant, PeerConfiguration};
use crate::common::task::task_resolver::{AdditionalTasks, TaskResolver};
use crate::common::task::TaskAbsent;
use crate::service::network_metrics::manager::NetworkMetricsManagerRef;
use crate::service::peer_configuration::NetworkInterfaceManagement;
use crate::service::tasks;


pub struct ServiceTaskResolver {
    peer_configuration: PeerConfiguration,
    network_interface_management: NetworkInterfaceManagement,
    metrics_manager: NetworkMetricsManagerRef,
}

impl ServiceTaskResolver {
    pub fn new(
        peer_configuration: PeerConfiguration,
        network_interface_management: NetworkInterfaceManagement,
        metrics_manager: NetworkMetricsManagerRef,
    ) -> Self {
        Self {
            peer_configuration,
            network_interface_management,
            metrics_manager,
        }
    }
}

impl TaskResolver for ServiceTaskResolver {
    fn resolve_tasks(&self, parameter: &ParameterVariant) -> Vec<Box<dyn TaskAbsent>> {
        let mut tasks: Vec<Box<dyn TaskAbsent>> = vec![];

        if let NetworkInterfaceManagement::Enabled { network_interface_manager, .. } = &self.network_interface_management {
            let network_interface_manager = network_interface_manager.clone();
            match parameter {
                ParameterVariant::DeviceInterface(_device_interface) => {
                    // TODO: Create task to check if device is present, if not, ignore it. Check parameters for CAN devices
                }
                ParameterVariant::EthernetBridge(ethernet_bridge) => {
                    tasks.push(Box::new(tasks::create_ethernet_bridge::CreateEthernetBridge { parameter: ethernet_bridge.value.clone(), network_interface_manager }));
                }
                ParameterVariant::Executor(_executor) => {
                    // TODO: Migrate to a task that can handle the executor
                }
                ParameterVariant::GreInterface(gre_interface) => {
                    tasks.push(Box::new(tasks::create_gre_interfaces::ManageGreInterface { parameter: gre_interface.value.clone(), network_interface_manager }));
                }
                ParameterVariant::JoinedInterface(joined_interface) => {
                    tasks.push(Box::new(tasks::manage_joined_interfaces::ManageJoinedInterface { parameter: joined_interface.value.clone(), network_interface_manager }));
                }
                ParameterVariant::RemotePeerConnectionCheck(_remote_peer_connection_check) => {
                    // TODO: Handle remote peer connection checks in its own tasks
                }
            };
        }

        tasks
    }

    fn additional_tasks(&self) -> Vec<AdditionalTasks> {

        let mut remote_peer_connection_checks = self.peer_configuration.remote_peer_connection_checks.clone().into_iter().collect::<Vec<_>>();
        remote_peer_connection_checks.sort_by(|a, b| a.target.cmp(&b.target));

        let remote_peers = remote_peer_connection_checks.into_iter()
            .map(|connection_check| (connection_check.value.remote_peer_id, connection_check.value.remote_ip))
            .collect::<HashMap<_, _>>(); //TODO split into multiple tasks
        let connection_check_task: Box<dyn TaskAbsent> = Box::new(tasks::setup_cluster_metrics::SetupClusterMetrics {
            remote_peers,
            metrics_manager: self.metrics_manager.clone(),
        });
        let connection_check_parameters = self.peer_configuration.remote_peer_connection_checks.clone().into_iter().collect::<Vec<_>>();
        if let Some(parameter) = connection_check_parameters.first() {
            let variant = ParameterVariant::RemotePeerConnectionCheck(Box::new(parameter.clone()));
            vec![AdditionalTasks {
                parameter: variant,
                tasks: Vec::from([connection_check_task]),
            }]
        } else {
            vec![]
        }
    }
}
