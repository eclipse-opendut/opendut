use std::collections::{HashMap, HashSet};
use std::net::Ipv4Addr;
use async_trait::async_trait;
use tracing::debug;
use opendut_types::peer::configuration::{parameter, Parameter};
use opendut_types::peer::configuration::parameter::GreAddresses;
use opendut_types::util::net::NetworkInterfaceName;
use crate::common::task::{Success, Task, TaskFulfilled};
use crate::service::network_interface::manager::interface::NetlinkInterfaceKind;
use crate::service::network_interface::manager::NetworkInterfaceManagerRef;

pub struct CreateGreInterfaces {
    pub parameter: Parameter<parameter::GreInterfaces>,
    pub network_interface_manager: NetworkInterfaceManagerRef,
}



#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct GreInterfaceAddressesWithIndex {
    pub local_ip: Ipv4Addr,
    pub remote_ip: Ipv4Addr,
    pub index: u32,
    pub name: NetworkInterfaceName,
}

#[async_trait]
impl Task for CreateGreInterfaces {
    fn description(&self) -> String {
        format!("Create GRE interfaces '{}'", self.parameter.value)
    }

    async fn check_fulfilled(&self) -> anyhow::Result<TaskFulfilled> {
        let interfaces = self.network_interface_manager.list_interfaces().await?;
        let gre_interfaces = interfaces.iter().filter_map(|interface| {
            if let NetlinkInterfaceKind::GreTap { local, remote } = interface.kind {
                let gre_with_index = GreInterfaceAddressesWithIndex {
                    local_ip: local,
                    remote_ip: remote,
                    index: interface.index,
                    name: interface.name.clone(),
                };
                let addresses = GreAddresses {
                    local_ip: local,
                    remote_ip: remote,
                };
                
                Some((addresses, gre_with_index))
            } else {
                None
            }
        }).collect::<HashMap<_, _>>();
        let gre_present_addresses = gre_interfaces.keys().cloned().collect::<HashSet<_>>();
        let gre_expected_addresses = self.parameter.value.address_list.iter().cloned().collect::<HashSet<_>>();
        
        let to_be_removed = gre_present_addresses.difference(&gre_expected_addresses).collect::<HashSet<_>>();
        let to_be_created = gre_expected_addresses.difference(&gre_present_addresses).collect::<HashSet<_>>();
        debug!("The following devices shall be removed: {:?}", to_be_removed);
        debug!("The following devices shall be created: {:?}", to_be_created);


        Ok(TaskFulfilled::Unchecked)
    }

    async fn execute(&self) -> anyhow::Result<Success> {
        // TODO: implement GRE creation / deletion
        
        Ok(Success::default())
    }
}


#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use std::sync::Arc;
    use uuid::Uuid;
    use opendut_types::peer::configuration::{ParameterId, ParameterTarget};
    use crate::common::task::runner;
    use crate::service::network_interface::manager::NetworkInterfaceManager;
    use crate::setup::RunMode;
    use super::*;

    #[test_with::env(RUN_EDGAR_NETLINK_INTEGRATION_TESTS)]
    #[test_log::test(tokio::test)]
    async fn test_create_gre_interfaces() -> anyhow::Result<()> {
        let fixture = Fixture::create();
        let parameter_present: Parameter<parameter::GreInterfaces> = Parameter::<parameter::GreInterfaces> {
            id: ParameterId(Uuid::new_v4()),
            dependencies: vec![],
            target: ParameterTarget::Present,
            value: parameter::GreInterfaces {
                address_list: vec![
                    GreAddresses { 
                        local_ip: Ipv4Addr::from_str("192.168.0.1")?, 
                        remote_ip: Ipv4Addr::from_str("192.168.0.2")?, 
                    },
                    GreAddresses {
                        local_ip: Ipv4Addr::from_str("192.168.0.1")?,
                        remote_ip: Ipv4Addr::from_str("192.168.0.3")?,
                    },
                    GreAddresses {
                        local_ip: Ipv4Addr::from_str("192.168.0.1")?,
                        remote_ip: Ipv4Addr::from_str("192.168.0.4")?,
                    }
                ],
            },
        };

        let tasks: Vec<Box<dyn Task>> = vec![
            Box::new(CreateGreInterfaces {
                parameter: parameter_present,
                network_interface_manager: Arc::clone(&fixture.network_interface_manager),
            })
        ];

        let result = runner::run(RunMode::Service, &tasks).await;
        assert!(result.is_ok());

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