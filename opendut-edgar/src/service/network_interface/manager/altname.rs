use crate::service::network_interface::manager::interface::Interface;
use crate::service::network_interface::manager::{Error, NetworkInterfaceManager};

pub const OPENDUT_ALTERNATIVE_INTERFACE_NAME_PREFIX: &str = "opendut";


impl NetworkInterfaceManager {
    pub async fn set_opendut_alternative_name(&self, interface: &Interface) -> Result<Interface, Error> {
        let alternative_names = [format!("{}-{}", OPENDUT_ALTERNATIVE_INTERFACE_NAME_PREFIX, interface.name.name())];
        let alt_names = alternative_names.iter().map(AsRef::as_ref).collect::<Vec<_>>();
        self.handle.link()
            .property_add(interface.index)
            .alt_ifname(&alt_names).execute()
            .await
            .map_err(|cause| Error::BridgeCreation { name: interface.name.clone(), cause: cause.into() })?;
        let interface = self.try_find_interface(&interface.name).await?;
        Ok(interface)
    }
}


#[cfg(test)]
mod tests {
    use opendut_model::util::net::NetworkInterfaceName;
    use super::*;

    #[test_with::env(RUN_EDGAR_NETLINK_INTEGRATION_TESTS)]
    #[test_log::test(tokio::test)]
    async fn test_set_opendut_alternative_name() -> anyhow::Result<()> {
        let (connection, handle, _) = rtnetlink::new_connection()?;
        tokio::spawn(connection);

        let manager = NetworkInterfaceManager { handle };
        let dummy = NetworkInterfaceName::try_from("dummy-42".to_string())?;
        let interface = manager.create_dummy_ipv4_interface(&dummy).await?;
        let interface = manager.set_opendut_alternative_name(&interface).await?;
        
        let opendut_alt_name = interface.alternative_names.into_iter().find(|interface| interface.starts_with(OPENDUT_ALTERNATIVE_INTERFACE_NAME_PREFIX));
        assert!(opendut_alt_name.is_some(), "Expected opendut alternative name was not set!");
        
        Ok(())
    }
}
