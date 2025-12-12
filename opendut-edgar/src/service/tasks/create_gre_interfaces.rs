use crate::common::task::{Success, Task, TaskAbsent, TaskStateFulfilled};
use crate::service::network_interface::manager::interface::Interface;
use crate::service::network_interface::manager::NetworkInterfaceManagerRef;
use async_trait::async_trait;
use rtnetlink::packet_route::link::LinkFlags;
use opendut_model::peer::configuration::parameter;

pub struct ManageGreInterface {
    pub parameter: parameter::GreInterfaceConfig,
    pub network_interface_manager: NetworkInterfaceManagerRef,
}

impl ManageGreInterface {
    async fn find_interface(&self) -> anyhow::Result<Option<Interface>> {
        let interface_name = self.parameter.interface_name()?;
        let interface = self.network_interface_manager.find_interface(&interface_name).await?;
        Ok(interface)
    }
}

#[async_trait]
impl Task for ManageGreInterface {
    fn description(&self) -> String {
        format!("Manage GRE interface '{}' to '{}'", self.parameter.local_ip, self.parameter.remote_ip)
    }
   
    async fn check_present(&self) -> anyhow::Result<TaskStateFulfilled> {
        let name = self.find_interface().await?;
        match name {
            Some(interface) => {
                let interface_is_up = interface.link_flags.contains(LinkFlags::Up);
                if interface_is_up {
                    Ok(TaskStateFulfilled::Yes)
                } else {
                    Ok(TaskStateFulfilled::No)
                }
            }
            None => {
                Ok(TaskStateFulfilled::No)
            }
        }        
    }

    async fn make_present(&self) -> anyhow::Result<Success> {
        let name = self.find_interface().await?;
        match name {
            None => {
                let name = self.parameter.interface_name()?;
                let interface = self.network_interface_manager.create_gretap_v4_interface(&name, &self.parameter.local_ip, &self.parameter.remote_ip).await?;
                self.network_interface_manager.set_interface_up(&interface).await?;
                self.network_interface_manager.set_opendut_alternative_name(&interface).await?;
            }
            Some(interface) => {
                self.network_interface_manager.set_interface_up(&interface).await?;
            }
        }
        Ok(Success::default())
    }
}

#[async_trait]
impl TaskAbsent for ManageGreInterface {
    async fn check_absent(&self) -> anyhow::Result<TaskStateFulfilled> {
        let name = self.find_interface().await?;
        match name {
            None => {
                Ok(TaskStateFulfilled::Yes)
            }
            Some(_) => {
                Ok(TaskStateFulfilled::No)
            }
        }
    }

    async fn make_absent(&self) -> anyhow::Result<Success> {
        let name = self.find_interface().await?;
        if let Some(interface) = name {
            self.network_interface_manager.delete_interface(&interface).await?;
        }
        Ok(Success::default())
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::service::network_interface::manager::NetworkInterfaceManager;
    use crate::service::tasks::runner::service_runner;
    use opendut_model::peer::configuration::ParameterTarget;
    use std::net::Ipv4Addr;
    use std::str::FromStr;
    use std::sync::Arc;

    #[test_with::env(RUN_EDGAR_NETLINK_INTEGRATION_TESTS)]
    #[test_log::test(tokio::test)]
    async fn test_create_gre_interface() -> anyhow::Result<()> {
        // ARRANGE
        let fixture = Fixture::create();
        let parameter = parameter::GreInterfaceConfig {
            local_ip: Ipv4Addr::from_str("192.168.0.1")?,
            remote_ip: Ipv4Addr::from_str("192.168.0.2")?,
        };
        let expected_name = parameter.interface_name()?;
        let gre_interface = fixture.network_interface_manager.find_interface(&expected_name).await?;
        assert!(gre_interface.is_none(), "GRE interface unexpectedly present!");

        // ACT
        let task: Box<dyn TaskAbsent> = Box::new(ManageGreInterface {
            parameter,
            network_interface_manager: Arc::clone(&fixture.network_interface_manager),
        });

        // ASSERT
        let result = service_runner::run_individual_task(task.as_ref(), ParameterTarget::Present).await;
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
