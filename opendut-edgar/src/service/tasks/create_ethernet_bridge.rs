use crate::common::task::{Success, Task, TaskAbsent, TaskStateFulfilled};
use crate::service::network_interface::manager::altname::OPENDUT_ALTERNATIVE_INTERFACE_NAME_PREFIX;
use crate::service::network_interface::manager::interface::NetlinkInterfaceKind;
use crate::service::network_interface::manager::NetworkInterfaceManagerRef;
use async_trait::async_trait;
use rtnetlink::packet_route::link::LinkFlags;
use opendut_model::peer::configuration::parameter;
use tracing::warn;

pub struct CreateEthernetBridge {
    pub parameter: parameter::EthernetBridge,
    pub network_interface_manager: NetworkInterfaceManagerRef,
}

#[async_trait]
impl Task for CreateEthernetBridge {
    fn description(&self) -> String {
        format!("Create bridge '{}'", self.parameter.name)
    }

    async fn check_present(&self) -> anyhow::Result<TaskStateFulfilled> {
        let interface = self.network_interface_manager.find_interface(&self.parameter.name).await?;
        match interface {
            Some(bridge) => {
                let interface_is_up = bridge.link_flags.contains(LinkFlags::Up);
                if NetlinkInterfaceKind::Bridge == bridge.kind && interface_is_up {
                    Ok(TaskStateFulfilled::Yes)
                } else {
                    Ok(TaskStateFulfilled::No)
                }
            },
            None => Ok(TaskStateFulfilled::No),
        }
    }

    async fn make_present(&self) -> anyhow::Result<Success> {
        let interface = self.network_interface_manager.find_interface(&self.parameter.name).await?;

        match interface {
            None => {
                let bridge = self.network_interface_manager.create_empty_bridge(&self.parameter.name).await?;
                self.network_interface_manager.set_opendut_alternative_name(&bridge).await?;
                self.network_interface_manager.set_interface_up(&bridge).await?;

                Ok(Success::default())
            }
            Some(bridge) => {
                if NetlinkInterfaceKind::Bridge == bridge.kind {
                    self.network_interface_manager.set_interface_up(&bridge).await?;
                    Ok(Success::default())
                } else {
                    Err(anyhow::Error::msg(format!("Another interface with that name exists but it has an unexpected interface kind: <{:?}>!", bridge.kind)))
                }

            }
        }
    }
}

#[async_trait]
impl TaskAbsent for CreateEthernetBridge {
    async fn check_absent(&self) -> anyhow::Result<TaskStateFulfilled> {
        let interface = self.network_interface_manager.find_interface(&self.parameter.name).await?;
        match interface {
            None => { Ok(TaskStateFulfilled::Yes) },
            Some(_) => { Ok(TaskStateFulfilled::No) },
        }
    }

    async fn make_absent(&self) -> anyhow::Result<Success> {
        let interface = self.network_interface_manager.find_interface(&self.parameter.name).await?;

        match interface {
            None => {
                Ok(Success::default())
            }
            Some(bridge) => {
                if NetlinkInterfaceKind::Bridge == bridge.kind {
                    let opendut_name = bridge.alternative_names.iter().find(|name| name.starts_with(OPENDUT_ALTERNATIVE_INTERFACE_NAME_PREFIX)).cloned();
                    match opendut_name {
                        None => {
                            warn!("Refusing to delete network bridge <{}> that was not created by openDuT!", bridge.name.name());
                        }
                        Some(_name) => {
                            self.network_interface_manager.delete_interface(&bridge).await?;
                        }
                    }
                    Ok(Success::default())
                } else {
                    Err(anyhow::Error::msg(format!("Another interface with that name exists but it has an unexpected interface kind: <{:?}>!", bridge.kind)))
                }
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::service::can::can_manager::CanManager;
    use crate::service::network_interface::manager::NetworkInterfaceManager;
    use crate::service::network_metrics::manager::{NetworkMetricsManager, NetworkMetricsOptions};
    use crate::service::peer_configuration::NetworkInterfaceManagement;
    use crate::common::task::service_runner;
    use opendut_model::peer::configuration::{parameter, ParameterTarget, PeerConfiguration};
    use opendut_model::util::net::NetworkInterfaceName;
    use rand::Rng;
    use std::sync::Arc;
    use crate::service::tasks;
    use crate::service::tasks::task_resolver::ServiceTaskResolver;

    #[test_with::env(RUN_EDGAR_NETLINK_INTEGRATION_TESTS)]
    #[test_log::test(tokio::test)]
    async fn create_bridge() -> anyhow::Result<()> {
        let fixture = Fixture::create();
        let result = fixture.network_interface_manager.find_interface(&fixture.bridge_name).await?;
        assert!(result.is_none());

        let result = service_runner::run_tasks(fixture.peer_configuration, fixture.service_task_resolver).await;
        assert!(result.items.len().eq(&1));

        let bridge_interface = fixture.network_interface_manager.find_interface(&fixture.bridge_name).await?;
        assert!(bridge_interface.is_some());
        fixture.network_interface_manager.delete_interface(&bridge_interface.unwrap()).await?;

        Ok(())
    }

    #[test_with::env(RUN_EDGAR_NETLINK_INTEGRATION_TESTS)]
    #[test_log::test(tokio::test)]
    async fn remove_bridge() -> anyhow::Result<()> {
        let fixture = Fixture::create();
        let task: Box<dyn TaskAbsent> = Box::new(CreateEthernetBridge {
            parameter: fixture.parameter,
            network_interface_manager: Arc::clone(&fixture.network_interface_manager),
        });
        let _bridge = fixture.network_interface_manager.create_empty_bridge(&fixture.bridge_name).await?;
        let interface = fixture.network_interface_manager.find_interface(&fixture.bridge_name).await?;
        assert!(interface.is_some(), "Expected a bridge but got none");
        let bridge = interface.unwrap();
        let _interface = fixture.network_interface_manager.set_opendut_alternative_name(&bridge).await?;

        service_runner::run_individual_task(task.as_ref(), ParameterTarget::Absent).await?;

        let bridge_interface = fixture.network_interface_manager.find_interface(&fixture.bridge_name).await?;
        assert!(bridge_interface.is_none(), "Expected that bridge was removed but it wasn't!");

        Ok(())
    }

    #[test_with::env(RUN_EDGAR_NETLINK_INTEGRATION_TESTS)]
    #[test_log::test(tokio::test)]
    async fn create_bridge_fails_if_another_interface_with_different_type_is_present() -> anyhow::Result<()> {
        let fixture = Fixture::create();
        let task: Box<dyn TaskAbsent> = Box::new(CreateEthernetBridge {
            parameter: fixture.parameter,
            network_interface_manager: Arc::clone(&fixture.network_interface_manager),
        });
        let _ = fixture.network_interface_manager.create_dummy_ipv4_interface(&fixture.bridge_name).await?;
        let another_interface = fixture.network_interface_manager.find_interface(&fixture.bridge_name).await?;
        assert!(another_interface.is_some());

        let result = service_runner::run_individual_task(task.as_ref(), ParameterTarget::Absent).await;

        assert!(result.is_err());
        fixture.network_interface_manager.delete_interface(&another_interface.unwrap()).await?;

        Ok(())
    }

    pub struct Fixture {
        bridge_name: NetworkInterfaceName,
        network_interface_manager: NetworkInterfaceManagerRef,
        peer_configuration: PeerConfiguration,
        parameter: parameter::EthernetBridge,
        service_task_resolver: ServiceTaskResolver,
    }
    impl Fixture {

        pub fn random_suffix() -> String {
            rand::rng()
                .sample_iter(&rand::distr::Alphanumeric)
                .take(7)
                .map(char::from)
                .collect()
        }
        pub fn create() -> Self {
            let (connection, handle, _) = rtnetlink::new_connection().expect("Could not get rtnetlink handle.");
            tokio::spawn(connection);
            let manager = NetworkInterfaceManager { handle };
            let network_interface_manager = Arc::new(manager);
            let suffix = Fixture::random_suffix();
            let bridge_name = NetworkInterfaceName::try_from(format!("dut-br-{suffix}")).unwrap();

            let parameter = parameter::EthernetBridge {
                name: bridge_name.clone(),
            };

            let mut peer_configuration = PeerConfiguration::default();
            peer_configuration.ethernet_bridges.set(
                parameter.clone(),
                ParameterTarget::Present,
                vec![],
            );
            let can_manager = CanManager::create(Arc::clone(&network_interface_manager));
            let network_interface_management = NetworkInterfaceManagement::Enabled {
                network_interface_manager: network_interface_manager.clone(),
                can_manager
            };
            let metrics_manager = NetworkMetricsManager::new(NetworkMetricsOptions::default());
            let service_task_resolver = tasks::task_resolver::ServiceTaskResolver::new(
                peer_configuration.clone(),
                network_interface_management.clone(),
                Arc::clone(&metrics_manager),
            );

            Self {
                bridge_name,
                network_interface_manager,
                peer_configuration,
                parameter,
                service_task_resolver,
            }
        }
    }
}
