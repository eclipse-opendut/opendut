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
