use crate::common::task::{Success, Task, TaskFulfilled};
use crate::service::network_interface::manager::interface::NetlinkInterfaceKind;
use crate::service::network_interface::manager::NetworkInterfaceManagerRef;
use async_trait::async_trait;
use netlink_packet_route::link::LinkFlag;
use tracing::{warn};
use opendut_types::peer::configuration::parameter;
use opendut_types::peer::configuration::{Parameter, ParameterTarget};
use crate::service::network_interface::manager::altname::OPENDUT_ALTERNATIVE_INTERFACE_NAME_PREFIX;

pub struct CreateEthernetBridge {
    pub parameter: Parameter<parameter::EthernetBridge>,
    pub network_interface_manager: NetworkInterfaceManagerRef,
}
#[async_trait]
impl Task for CreateEthernetBridge {
    fn description(&self) -> String {
        format!("Create bridge '{}'", self.parameter.value.name)
    }

    async fn check_fulfilled(&self) -> anyhow::Result<TaskFulfilled> {
        let interface = self.network_interface_manager.find_interface(&self.parameter.value.name).await?;

        match self.parameter.target {
            ParameterTarget::Present => {
                match interface {
                    Some(bridge) => {
                        let interface_is_up = bridge.link_flag.contains(&LinkFlag::Up);
                        if NetlinkInterfaceKind::Bridge == bridge.kind && interface_is_up {
                            Ok(TaskFulfilled::Yes)
                        } else {
                            Ok(TaskFulfilled::No)
                        }
                    },
                    None => Ok(TaskFulfilled::No),
                }
            },
            ParameterTarget::Absent => {
                match interface {
                    None => { Ok(TaskFulfilled::Yes) },
                    Some(_) => { Ok(TaskFulfilled::No) },
                }
            }
        }
    }

    async fn execute(&self) -> anyhow::Result<Success> {
        let interface = self.network_interface_manager.find_interface(&self.parameter.value.name).await?;

        match self.parameter.target {
            ParameterTarget::Present => {
                match interface {
                    None => {
                        let bridge = self.network_interface_manager.create_empty_bridge(&self.parameter.value.name).await?;
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
            ParameterTarget::Absent => {
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
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use rand::Rng;
    use uuid::Uuid;
    use opendut_types::peer::configuration::parameter;
    use opendut_types::peer::configuration::ParameterId;
    use opendut_types::util::net::NetworkInterfaceName;
    use crate::common::task::runner;
    use crate::service::network_interface::manager::NetworkInterfaceManager;
    use crate::setup::RunMode;
    use super::*;

    #[test_with::env(RUN_EDGAR_NETLINK_INTEGRATION_TESTS)]
    #[test_log::test(tokio::test)]
    async fn create_bridge() -> anyhow::Result<()> {
        let fixture = Fixture::create();
        let tasks: Vec<Box<dyn Task>> = vec![
            Box::new(CreateEthernetBridge {
                parameter: fixture.parameter_present,
                network_interface_manager: Arc::clone(&fixture.network_interface_manager),
            })
        ];
        let result = fixture.network_interface_manager.find_interface(&fixture.bridge_name).await?;
        assert!(result.is_none());

        runner::run(RunMode::Service, &tasks).await?;

        let bridge_interface = fixture.network_interface_manager.find_interface(&fixture.bridge_name).await?;
        assert!(bridge_interface.is_some());
        fixture.network_interface_manager.delete_interface(&bridge_interface.unwrap()).await?;

        Ok(())
    }

    #[test_with::env(RUN_EDGAR_NETLINK_INTEGRATION_TESTS)]
    #[test_log::test(tokio::test)]
    async fn remove_bridge() -> anyhow::Result<()> {
        let fixture = Fixture::create();
        let tasks: Vec<Box<dyn Task>> = vec![
            Box::new(CreateEthernetBridge {
                parameter: fixture.parameter_absent,
                network_interface_manager: Arc::clone(&fixture.network_interface_manager),
            })
        ];
        let _bridge = fixture.network_interface_manager.create_empty_bridge(&fixture.bridge_name).await?;
        let interface = fixture.network_interface_manager.find_interface(&fixture.bridge_name).await?;
        assert!(interface.is_some(), "Expected a bridge but got none");
        let bridge = interface.unwrap();
        let _interface = fixture.network_interface_manager.set_opendut_alternative_name(&bridge).await?;

        runner::run(RunMode::Service, &tasks).await?;

        let bridge_interface = fixture.network_interface_manager.find_interface(&fixture.bridge_name).await?;
        assert!(bridge_interface.is_none(), "Expected that bridge was removed but it wasn't!");

        Ok(())
    }

    #[test_with::env(RUN_EDGAR_NETLINK_INTEGRATION_TESTS)]
    #[test_log::test(tokio::test)]
    async fn create_bridge_fails_if_another_interface_with_different_type_is_present() -> anyhow::Result<()> {
        let fixture = Fixture::create();
        let tasks: Vec<Box<dyn Task>> = vec![
            Box::new(CreateEthernetBridge {
                parameter: fixture.parameter_present,
                network_interface_manager: Arc::clone(&fixture.network_interface_manager),
            })
        ];
        let _ = fixture.network_interface_manager.create_dummy_ipv4_interface(&fixture.bridge_name).await?;
        let another_interface = fixture.network_interface_manager.find_interface(&fixture.bridge_name).await?;
        assert!(another_interface.is_some());

        let result = runner::run(RunMode::Service, &tasks).await;

        assert!(result.is_err());
        fixture.network_interface_manager.delete_interface(&another_interface.unwrap()).await?;

        Ok(())
    }

    pub struct Fixture {
        bridge_name: NetworkInterfaceName,
        network_interface_manager: NetworkInterfaceManagerRef,
        parameter_present: Parameter<parameter::EthernetBridge>,
        parameter_absent: Parameter<parameter::EthernetBridge>,
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
            let bridge_name = NetworkInterfaceName::try_from(format!("dut-br-{}", suffix)).unwrap();

            let parameter_present: Parameter<parameter::EthernetBridge> = Parameter::<parameter::EthernetBridge> {
                id: ParameterId(Uuid::new_v4()),
                dependencies: vec![],
                target: ParameterTarget::Present,
                value: parameter::EthernetBridge {
                    name: bridge_name.clone(),
                },
            };

            let mut parameter_absent: Parameter<parameter::EthernetBridge> = parameter_present.clone();
            parameter_absent.id = ParameterId(Uuid::new_v4());
            parameter_absent.target = ParameterTarget::Absent;

            Self {
                bridge_name,
                network_interface_manager,
                parameter_present,
                parameter_absent,
            }
        }
    }
}