use futures::TryStreamExt;
use rtnetlink::{LinkBridge, LinkMessageBuilder};
use tracing::warn;
use opendut_model::util::net::NetworkInterfaceName;
use crate::service::network_interface::manager::interface::Interface;
use crate::service::network_interface::manager::{Error, NetworkInterfaceManager};
use crate::service::network_interface::manager::list_joined_interfaces::ShowJoinedInterfaces;

impl NetworkInterfaceManager {
    pub async fn find_interfaces_joined_to_bridge(&self, name: &NetworkInterfaceName) -> Result<Vec<Interface>, Error> {
        let bridge_name = self.find_interface(name).await?;
        match bridge_name {
            None => {
                Ok(Vec::new())
            }
            Some(interface) => {
                let interfaces = self.handle.link()
                    .get()
                    .filter_interfaces_joined_to(interface.index)
                    .execute()
                    .try_collect::<Vec<_>>().await
                    .map_err(|cause| Error::ListInterfaces { cause: cause.into() })?
                    .into_iter()
                    .filter_map(|link_message| {
                        let index = link_message.header.index;
                        Interface::try_from(link_message)
                            .inspect_err(|cause| warn!("Could not determine attributes of interface with index '{index}': {cause}"))
                            .ok()
                    })
                    .collect::<Vec<_>>();
                Ok(interfaces)
            }
        }
    }

    pub async fn join_interface_to_bridge(&self, interface: &Interface, bridge: &Interface) -> Result<(), Error> {
        self.handle
            .link()
            .set(
                LinkMessageBuilder::<LinkBridge>::default()
                    .index(interface.index)
                    .controller(bridge.index)
                    .build()
            )
            .execute().await
            .map_err(|cause| Error::JoinInterfaceToBridge {
                interface: Box::new(interface.clone()),
                bridge: Box::new(bridge.clone()),
                cause: cause.into()
            })?;
        Ok(())
    }

    pub async fn remove_interface_from_bridge(&self, interface: &Interface) -> Result<(), Error> {
        self.handle
            .link()
            .set(
                LinkMessageBuilder::<LinkBridge>::default()
                    .index(interface.index)
                    .nocontroller()
                    .build()
            )
            .execute().await
            .map_err(|cause| Error::ModificationFailure { name: interface.name.clone(), cause: format!("Failed to remove controller from interface. {cause}") })?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use tracing::debug;
    use opendut_model::util::net::NetworkInterfaceName;
    use crate::service::network_interface::manager::NetworkInterfaceManager;

    #[test_with::env(RUN_EDGAR_NETLINK_INTEGRATION_TESTS)]
    #[test_log::test(tokio::test)]
    async fn test_create_bridge() -> anyhow::Result<()> {
        let (connection, handle, _) = rtnetlink::new_connection().unwrap();
        tokio::spawn(connection);

        let manager = NetworkInterfaceManager { handle };
        let bridge_name = NetworkInterfaceName::try_from("bridge-test".to_string())?;
        let bridge = manager.create_empty_bridge(&bridge_name).await?;
        let bridge = manager.set_opendut_alternative_name(&bridge).await?;
        let expected_alternative_name = format!("opendut-{bridge_name}");
        let dummy1_name = NetworkInterfaceName::try_from("dummy-test1".to_string())?;
        let dummy2_name = NetworkInterfaceName::try_from("dummy-test2".to_string())?;
        let dummy1 = manager.create_dummy_ipv4_interface(&dummy1_name).await?;
        let dummy2 = manager.create_dummy_ipv4_interface(&dummy2_name).await?;
        manager.join_interface_to_bridge(&dummy1, &bridge).await?;
        manager.join_interface_to_bridge(&dummy2, &bridge).await?;
        assert!(!bridge.alternative_names.is_empty(), "Expected bridge alternative name was not set!");
        assert!(bridge.alternative_names.contains(&expected_alternative_name), "Bridge alternative name does not contain opendut!");

        let child_interfaces = manager.find_interfaces_joined_to_bridge(&bridge_name).await?;
        let child_interface_names = child_interfaces.iter().map(|interface| interface.name.name()).collect::<Vec<_>>();
        assert_eq!(child_interfaces.len(), 2);
        assert!(child_interface_names.contains(&String::from("dummy-test1")));
        assert!(child_interface_names.contains(&String::from("dummy-test2")));

        manager.remove_interface_from_bridge(&dummy2).await?;
        let child_interfaces = manager.find_interfaces_joined_to_bridge(&bridge_name).await?;

        assert_eq!(child_interfaces.len(), 1);
        assert_eq!(child_interfaces[0].name.name(), "dummy-test1");
        debug!("{:?}", child_interfaces);
        Ok(())
    }
}
