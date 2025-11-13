use crate::common::task::task_resolver::{AdditionalTasks, TaskResolver};
use crate::common::task::TaskAbsent;
use crate::service::network_metrics::manager::NetworkMetricsManagerRef;
use crate::service::peer_configuration::NetworkInterfaceManagement;
use crate::service::tasks;
use opendut_model::peer::configuration::{ParameterVariant, PeerConfiguration};
use std::collections::HashMap;
use opendut_model::peer::configuration::parameter::DeviceInterface;
use opendut_model::util::net::NetworkInterfaceConfiguration;
use crate::service::network_interface::manager::can::CanInterfaceConfiguration;
use crate::service::network_interface::manager::NetworkInterfaceManagerRef;

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

        if let NetworkInterfaceManagement::Enabled { network_interface_manager, can_manager } = &self.network_interface_management {
            let network_interface_manager = network_interface_manager.clone();
            match parameter {
                ParameterVariant::DeviceInterface(device_interface) => {
                    resolve_device_interface_tasks(&device_interface.value, network_interface_manager, &mut tasks);
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
                ParameterVariant::CanConnections(parameter) => {
                    tasks.push(Box::new(tasks::can_connection::CanConnection { parameter: parameter.value.clone(), can_manager: can_manager.clone() }));
                }
                ParameterVariant::CanBridges(parameter) => {
                    tasks.push(Box::new(tasks::can_virtual_device::CanCreateVirtualDevice { name: parameter.value.name.clone(), network_interface_manager }));
                }
                ParameterVariant::CanLocalRoutes(parameter) => {
                    tasks.push(Box::new(tasks::can_local_route::CanLocalRoute { parameter: parameter.value.clone(), network_interface_manager: network_interface_manager.clone(), can_fd: false }));
                    tasks.push(Box::new(tasks::can_local_route::CanLocalRoute { parameter: parameter.value.clone(), network_interface_manager, can_fd: true }));
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

fn resolve_device_interface_tasks(
    device_interface: &DeviceInterface,
    network_interface_manager: NetworkInterfaceManagerRef,
    tasks: &mut Vec<Box<dyn TaskAbsent>>,
) {
    match device_interface.descriptor.configuration.clone() {
        NetworkInterfaceConfiguration::Ethernet => {
            // TODO: Create task to check if device is present, if not, ignore it.
        }
        NetworkInterfaceConfiguration::Can { bitrate, sample_point, fd, data_bitrate, data_sample_point } => {
            // TODO: Create task to check if device is present, if not, ignore it.

            let can_config = CanInterfaceConfiguration::new(bitrate, sample_point, fd, data_bitrate, data_sample_point);
            let can_config_task = tasks::can_device_configuration::CanDeviceConfiguration {
                interface_name: device_interface.descriptor.name.clone(),
                can_config,
                network_interface_manager,
            };

            tasks.push(Box::new(can_config_task));
        }
        NetworkInterfaceConfiguration::Vcan => {
            tasks.push(Box::new(tasks::can_virtual_device::CanCreateVirtualDevice {
                name: device_interface.descriptor.name.clone(),
                network_interface_manager
            }));
        }
    }
}
