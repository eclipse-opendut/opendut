use std::fmt::{Debug, Formatter};
use std::io;
use std::net::Ipv4Addr;

use anyhow::anyhow;
use futures::TryStreamExt;
use netlink_packet_route::link::nlas;
use netlink_packet_route::LinkMessage;

use opendut_types::topology::InterfaceName;

use crate::service::network_device::gretap::Gretap;

pub struct NetworkDeviceManager {
    handle: rtnetlink::Handle,
}
impl NetworkDeviceManager {
    pub fn create() -> Result<Self> {
        let (connection, handle, _) = rtnetlink::new_connection()
            .map_err(|cause| Error::Connecting { cause })?;
        tokio::spawn(connection);

        Ok(Self { handle })
    }

    pub async fn list_interfaces(&self) -> Result<Vec<Interface>> {
        fn interface_name_from(interface: LinkMessage) -> anyhow::Result<InterfaceName> {
            let interface_name = interface.nlas.into_iter()
                .find_map(|nla| match nla {
                    nlas::Nla::IfName(name) => Some(name),
                    _ => None,
                })
                .ok_or(anyhow!("No name attribute found."))?;
            let interface_name = InterfaceName::try_from(interface_name)?;
            Ok(interface_name)
        }

        let interfaces = self.handle
            .link()
            .get()
            .execute()
            .try_collect::<Vec<_>>().await
            .map_err(|cause| Error::ListInterfaces { cause })?
            .into_iter()
            .filter_map(|interface| {
                let index = interface.header.index;
                match interface_name_from(interface) {
                    Ok(name) => Some(Interface { index, name }),
                    Err(cause) => {
                        log::warn!("Could not determine name of interface with index '{index}': {cause}");
                        None
                    }
                }
            })
            .collect::<Vec<_>>();
        Ok(interfaces)
    }

    pub async fn find_interface(&self, name: &InterfaceName) -> Result<Option<Interface>> {
        let interfaces = self.list_interfaces().await?;
        let maybe_interface = interfaces.into_iter().find(|interface| interface.name == *name);
        Ok(maybe_interface)
    }
    pub async fn try_find_interface(&self, name: &InterfaceName) -> Result<Interface> {
        self.find_interface(name).await?
            .ok_or(Error::InterfaceNotFound { name: name.clone() })
    }

    pub async fn create_empty_bridge(&self, name: &InterfaceName) -> Result<Interface> {
        self.handle
            .link()
            .add()
            .bridge(name.name())
            .execute().await
            .map_err(|cause| Error::BridgeCreation { name: name.clone(), cause })?;
        let interface = self.try_find_interface(name).await?;
        Ok(interface)
    }

    // We only support IPv4 for now, as NetBird only assigns IPv4 addresses to peers.
    // This does not prevent IPv6 traffic from being routed between peers.
    pub async fn create_gretap_v4_interface(&self, name: &InterfaceName, local_ip: &Ipv4Addr, remote_ip: &Ipv4Addr) -> Result<Interface> {
        self.handle
            .link()
            .add()
            .gretap_v4(name.name(), local_ip, remote_ip)
            .execute().await
            .map_err(|cause| Error::GretapCreation { name: name.clone(), cause })?;
        let interface = self.try_find_interface(name).await?;
        Ok(interface)
    }

    pub async fn set_interface_up(&self, interface: &Interface) -> Result<()> {
        self.handle
            .link()
            .set(interface.index)
            .up()
            .execute().await
            .map_err(|cause| Error::SetInterfaceUp { interface: interface.clone(), cause })?;
        Ok(())
    }

    pub async fn get_attributes(&self, interface: &Interface) -> Result<Vec<nlas::Nla>> {
        let interface_list = self.handle
            .link()
            .get()
            .match_index(interface.index)
            .execute()
            .try_collect::<Vec<_>>().await
            .map_err(|cause| Error::ListInterfaces { cause })?;

        let nlas = interface_list
            .first() //only one should match index
            .ok_or(Error::InterfaceNotFound { name: interface.name.clone() })?
            .nlas.clone();

        Ok(nlas)
    }

    pub async fn join_interface_to_bridge(&self, interface: &Interface, bridge: &Interface) -> Result<()> {
        self.handle
            .link()
            .set(interface.index)
            .master(bridge.index)
            .execute().await
            .map_err(|cause| Error::JoinInterfaceToBridge { interface: interface.clone(), bridge: bridge.clone(), cause })?;
        Ok(())
    }

    pub async fn delete_interface(&self, interface: &Interface) -> Result<()> {
        self.handle
            .link()
            .del(interface.index)
            .execute().await
            .map_err(|cause| Error::DeleteInterface { interface: interface.clone(), cause })?;
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct Interface {
    pub index: u32,
    pub name: InterfaceName,
}
impl std::fmt::Display for Interface {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}: {}]", self.index, self.name)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failure while creating bridge '{name}': {cause}")]
    BridgeCreation { name: InterfaceName, cause: rtnetlink::Error },
    #[error("Failed to establish connection to netlink: {cause}")]
    Connecting { cause: io::Error },
    #[error("Failure while deleting interface {interface}: {cause}")]
    DeleteInterface { interface: Interface, cause: rtnetlink::Error },
    #[error("Failure while creating gretap interface '{name}': {cause}")]
    GretapCreation { name: InterfaceName, cause: rtnetlink::Error },
    #[error("Interface with name '{name}' not found.")]
    InterfaceNotFound { name: InterfaceName },
    #[error("Failure while listing interfaces: {cause}")]
    ListInterfaces { cause: rtnetlink::Error },
    #[error("Failure while setting interface {interface} to state 'up': {cause}")]
    SetInterfaceUp { interface: Interface, cause: rtnetlink::Error },
    #[error("Failure while joining interface {interface} to bridge {bridge}: {cause}")]
    JoinInterfaceToBridge { interface: Interface, bridge: Interface, cause: rtnetlink::Error },
    #[error("{message}")]
    Other { message: String },
}
pub type Result<T> = std::result::Result<T, Error>;
