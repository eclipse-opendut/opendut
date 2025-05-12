use std::collections::{HashMap, HashSet};
use std::net::Ipv4Addr;
use async_trait::async_trait;
use tracing::{trace, warn};
use opendut_types::peer::configuration::{parameter, Parameter};
use opendut_types::peer::configuration::parameter::GreAddresses;
use opendut_types::util::net::NetworkInterfaceName;
use crate::common::task::{Success, Task, TaskFulfilled};
use crate::service::network_interface::manager::interface::{NetlinkInterfaceKind};
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

#[derive(Clone, Debug)]
pub struct GreInterfaceConfigurationChanges {
    gre_expected_interfaces: HashSet<GreAddresses>,
    to_be_removed: HashMap<GreAddresses, GreInterfaceAddressesWithIndex>,
    to_be_created: HashSet<GreAddresses>,
}
impl std::fmt::Display for GreInterfaceConfigurationChanges {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        
        f.write_fmt(format_args!(
                "The following devices need to be removed: {:?}. \
                 The following devices need to be created: {:?}", self.to_be_removed, self.to_be_created))
    }
}

impl GreInterfaceConfigurationChanges {
    async fn determine(parameter: &Parameter<parameter::GreInterfaces>, network_interface_manager: NetworkInterfaceManagerRef) -> anyhow::Result<Self> {
        let interfaces = network_interface_manager.list_interfaces().await?;
        let gre_interfaces = interfaces.iter().filter_map(|interface| {
            if interface.name.name() == "gretap0" {
                None
            } else if let NetlinkInterfaceKind::GreTap { local, remote } = interface.kind {
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
        let gre_present_interfaces = gre_interfaces.keys().cloned().collect::<HashSet<_>>();
        let gre_expected_interfaces = parameter.value.address_list.iter().cloned().collect::<HashSet<_>>();

        let to_be_removed = gre_interfaces.iter()
            .flat_map(|(gre_address, gre_address_with_index)| { 
                if !gre_expected_interfaces.contains(gre_address) {
                    Some((gre_address.clone(), gre_address_with_index.clone()))
                } else {
                    None
                }
            })
            .collect::<HashMap<GreAddresses, GreInterfaceAddressesWithIndex>>();
        let to_be_created = gre_expected_interfaces.difference(&gre_present_interfaces).cloned().collect::<HashSet<_>>();
        
        Ok(Self {
            gre_expected_interfaces,
            to_be_removed,
            to_be_created,
        })
    }
    
    fn done(&self) -> bool {
        self.to_be_removed.is_empty() && self.to_be_created.is_empty()
    }
}


#[async_trait]
impl Task for CreateGreInterfaces {
    fn description(&self) -> String {
        format!("Create GRE interfaces '{}'", self.parameter.value)
    }

    async fn check_fulfilled(&self) -> anyhow::Result<TaskFulfilled> {
        let changes = GreInterfaceConfigurationChanges::determine(&self.parameter, self.network_interface_manager.clone()).await?;
        trace!("Following changes necessary: {}", changes);
        if changes.done() {
            Ok(TaskFulfilled::Yes)
        } else {
            Ok(TaskFulfilled::No)
        }
    }

    async fn execute(&self) -> anyhow::Result<Success> {
        let changes = GreInterfaceConfigurationChanges::determine(&self.parameter, self.network_interface_manager.clone()).await?;
        for add_gre_interface in changes.to_be_created {
            let name = add_gre_interface.interface_name()?;
            let interface = self.network_interface_manager.create_gretap_v4_interface(&name, &add_gre_interface.local_ip, &add_gre_interface.remote_ip).await?;
            self.network_interface_manager.set_opendut_alternative_name(&interface).await?;
        }
        for (_, remove_gre_interface) in changes.to_be_removed {
            let interface = self.network_interface_manager.find_interface(&remove_gre_interface.name).await?;
            match interface {
                Some(interface) => {
                    self.network_interface_manager.delete_interface(&interface).await?;
                }
                None => {
                    warn!("Could not find GRE interface '{}' for deletion.", remove_gre_interface.name);
                }
            }

        }

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
                parameter: parameter_present.clone(),
                network_interface_manager: Arc::clone(&fixture.network_interface_manager),
            })
        ];

        let result = runner::run(RunMode::Service, &tasks).await;
        assert!(result.is_ok());
        
        
        let changes = GreInterfaceConfigurationChanges::determine(&parameter_present, fixture.network_interface_manager.clone()).await?;
        assert!(changes.done());
        

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