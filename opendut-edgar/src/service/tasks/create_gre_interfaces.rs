use crate::common::task::{Success, Task, TaskFulfilled};
use crate::service::network_interface::manager::NetworkInterfaceManagerRef;
use async_trait::async_trait;
use netlink_packet_route::link::LinkFlag;
use opendut_types::peer::configuration::{parameter, Parameter, ParameterTarget};

pub struct ManageGreInterface {
    pub parameter: Parameter<parameter::GreInterfaceConfig>,
    pub network_interface_manager: NetworkInterfaceManagerRef,
}


#[async_trait]
impl Task for ManageGreInterface {
    fn description(&self) -> String {
        format!("Manage GRE interface '{}'", self.parameter.value)
    }

    async fn check_fulfilled(&self) -> anyhow::Result<TaskFulfilled> {
        let interface_name = self.parameter.value.interface_name()?;
        let name = self.network_interface_manager.find_interface(&interface_name).await?;
        match (name, self.parameter.target) {
            (Some(_), ParameterTarget::Absent) | (None, ParameterTarget::Present) => {
                Ok(TaskFulfilled::No)
            }
            (Some(interface), ParameterTarget::Present) => {
                let interface_is_up = interface.link_flag.contains(&LinkFlag::Up);
                if interface_is_up {
                    Ok(TaskFulfilled::Yes)
                } else {
                    Ok(TaskFulfilled::No)
                }
            }
            (None, ParameterTarget::Absent) => {
                Ok(TaskFulfilled::Yes)
            }
        }        
    }

    async fn execute(&self) -> anyhow::Result<Success> {
        let interface_name = self.parameter.value.interface_name()?;
        let name = self.network_interface_manager.find_interface(&interface_name).await?;
        match (name, self.parameter.target) {
            (Some(name), ParameterTarget::Absent) => {
                self.network_interface_manager.delete_interface(&name).await?;
            }
            (None, ParameterTarget::Present) => {
                let name = self.parameter.value.interface_name()?;
                let interface = self.network_interface_manager.create_gretap_v4_interface(&name, &self.parameter.value.local_ip, &self.parameter.value.remote_ip).await?;
                self.network_interface_manager.set_interface_up(&interface).await?;
                self.network_interface_manager.set_opendut_alternative_name(&interface).await?;
            }
            (Some(interface), ParameterTarget::Present) => {
                self.network_interface_manager.set_interface_up(&interface).await?;
            }
            (None, ParameterTarget::Absent) => {
                // nothing to do
            }
        }
        Ok(Success::default())
    }
}


#[cfg(test)]
mod tests {
    use std::net::Ipv4Addr;
    use super::*;
    use crate::common::task::runner;
    use crate::service::network_interface::manager::NetworkInterfaceManager;
    use crate::setup::RunMode;
    use opendut_types::peer::configuration::{ParameterId, ParameterTarget};
    use std::str::FromStr;
    use std::sync::Arc;
    use uuid::Uuid;

    #[test_with::env(RUN_EDGAR_NETLINK_INTEGRATION_TESTS)]
    #[test_log::test(tokio::test)]
    async fn test_create_gre_interfaces() -> anyhow::Result<()> {
        // ARRANGE
        let fixture = Fixture::create();
        let parameter = parameter::GreInterfaceConfig {
            local_ip: Ipv4Addr::from_str("192.168.0.1")?,
            remote_ip: Ipv4Addr::from_str("192.168.0.2")?,
        };
        let expected_name = parameter.interface_name()?;
        let parameter_present: Parameter<parameter::GreInterfaceConfig> = Parameter::<parameter::GreInterfaceConfig> {
            id: ParameterId(Uuid::new_v4()),
            dependencies: vec![],
            target: ParameterTarget::Present,
            value: parameter,
        };
        let gre_interface = fixture.network_interface_manager.find_interface(&expected_name).await?;
        assert!(gre_interface.is_none(), "GRE interface unexpectedly present!");

        // ACT
        let tasks: Vec<Box<dyn Task>> = vec![
            Box::new(ManageGreInterface {
                parameter: parameter_present.clone(),
                network_interface_manager: Arc::clone(&fixture.network_interface_manager),
            })
        ];

        // ASSERT
        let result = runner::run(RunMode::Service, &tasks).await;
        assert!(result.is_ok());
        let gre_interface = fixture.network_interface_manager.find_interface(&expected_name).await?;
        assert!(gre_interface.is_some(), "GRE interface not found!");
        
        Ok(())
    }

    pub struct Fixture {
        network_interface_manager: NetworkInterfaceManagerRef,
    }

    impl Fixture {
        pub fn create() -> Self {
            let (connection, handle, _) = rtnetlink::new_connection().expect("Could not get rtnetlink handle.");
            tokio::spawn(connection);
            let manager = NetworkInterfaceManager { handle };
            let network_interface_manager = Arc::new(manager);

            Self {
                network_interface_manager,
            }
        }
    }

}