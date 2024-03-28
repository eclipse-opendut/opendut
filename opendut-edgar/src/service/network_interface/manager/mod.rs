use std::fmt::{Debug, Formatter};
use std::io;
use std::net::Ipv4Addr;
use std::sync::Arc;

use anyhow::anyhow;
use futures::TryStreamExt;
use netlink_packet_route::link::{LinkAttribute, LinkMessage};
use tokio::process::Command;

use gretap::Gretap;
use opendut_types::util::net::NetworkInterfaceName;

mod gretap;

pub type NetworkInterfaceManagerRef = Arc<NetworkInterfaceManager>;

pub struct NetworkInterfaceManager {
    handle: rtnetlink::Handle,
}
impl NetworkInterfaceManager {
    pub fn create() -> Result<NetworkInterfaceManagerRef, Error> {
        let (connection, handle, _) = rtnetlink::new_connection()
            .map_err(|cause| Error::Connecting { cause })?;
        tokio::spawn(connection);

        Ok(Arc::new(Self { handle }))
    }

    pub async fn list_interfaces(&self) -> Result<Vec<Interface>, Error> {
        fn interface_name_from(interface: LinkMessage) -> anyhow::Result<NetworkInterfaceName> {
            let interface_name = interface.attributes.into_iter()
                .find_map(|nla| match nla {
                    LinkAttribute::IfName(name) => Some(name),
                    _ => None,
                })
                .ok_or(anyhow!("No name attribute found."))?;
            let interface_name = NetworkInterfaceName::try_from(interface_name)?;
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

    pub async fn find_interface(&self, name: &NetworkInterfaceName) -> Result<Option<Interface>, Error> {
        let interfaces = self.list_interfaces().await?;
        let maybe_interface = interfaces.into_iter().find(|interface| interface.name == *name);
        Ok(maybe_interface)
    }
    pub async fn try_find_interface(&self, name: &NetworkInterfaceName) -> Result<Interface, Error> {
        self.find_interface(name).await?
            .ok_or(Error::InterfaceNotFound { name: name.clone() })
    }

    pub async fn create_empty_bridge(&self, name: &NetworkInterfaceName) -> Result<Interface, Error> {
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
    pub async fn create_gretap_v4_interface(&self, name: &NetworkInterfaceName, local_ip: &Ipv4Addr, remote_ip: &Ipv4Addr) -> Result<Interface, Error> {
        self.handle
            .link()
            .add()
            .gretap_v4(name.name(), local_ip, remote_ip)
            .execute().await
            .map_err(|cause| Error::GretapCreation { name: name.clone(), cause })?;
        let interface = self.try_find_interface(name).await?;
        Ok(interface)
    }

    pub async fn set_interface_up(&self, interface: &Interface) -> Result<(), Error> {
        self.handle
            .link()
            .set(interface.index)
            .up()
            .execute().await
            .map_err(|cause| Error::SetInterfaceUp { interface: interface.clone(), cause })?;
        Ok(())
    }

    pub async fn get_attributes(&self, interface: &Interface) -> Result<Vec<LinkAttribute>, Error> {
        let interface_list = self.handle
            .link()
            .get()
            .match_index(interface.index)
            .execute()
            .try_collect::<Vec<_>>().await
            .map_err(|cause| Error::ListInterfaces { cause })?;

        let attributes = interface_list
            .first() //only one should match index
            .ok_or(Error::InterfaceNotFound { name: interface.name.clone() })?
            .attributes.clone();

        Ok(attributes)
    }

    pub async fn join_interface_to_bridge(&self, interface: &Interface, bridge: &Interface) -> Result<(), Error> {
        self.handle
            .link()
            .set(interface.index)
            .controller(bridge.index)
            .execute().await
            .map_err(|cause| Error::JoinInterfaceToBridge { interface: interface.clone(), bridge: bridge.clone(), cause })?;
        Ok(())
    }

    pub async fn delete_interface(&self, interface: &Interface) -> Result<(), Error> {
        self.handle
            .link()
            .del(interface.index)
            .execute().await
            .map_err(|cause| Error::DeleteInterface { interface: interface.clone(), cause })?;
        Ok(())
    }

    pub async fn create_vcan_interface(&self, name: &NetworkInterfaceName) -> Result<Interface, Error> {
        let output = Command::new("ip")
                .arg("link")
                .arg("add")
                .arg("dev")
                .arg(name.name())
                .arg("type")
                .arg("vcan")
                .output()
                .await
                .map_err(|cause| Error::CommandLineProgramExecution { command: "cangw".to_string(), cause })?;
        
        if ! output.status.success() {
            return Err(Error::VCanInterfaceCreation { name: name.clone(), cause: format!("{:?}", String::from_utf8_lossy(&output.stderr).trim()) });
        }

        let interface = self.try_find_interface(name).await?;
        Ok(interface)
    }
    
}

#[derive(Clone, Debug)]
pub struct Interface {
    pub index: u32,
    pub name: NetworkInterfaceName,
}
impl std::fmt::Display for Interface {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}: {}]", self.index, self.name)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failure while creating bridge '{name}': {cause}")]
    BridgeCreation { name: NetworkInterfaceName, cause: rtnetlink::Error },
    #[error("Failed to establish connection to netlink: {cause}")]
    Connecting { cause: io::Error },
    #[error("Failure while deleting interface {interface}: {cause}")]
    DeleteInterface { interface: Interface, cause: rtnetlink::Error },
    #[error("Failure while creating gretap interface '{name}': {cause}")]
    GretapCreation { name: NetworkInterfaceName, cause: rtnetlink::Error },
    #[error("Interface with name '{name}' not found.")]
    InterfaceNotFound { name: NetworkInterfaceName },
    #[error("Failure while listing interfaces: {cause}")]
    ListInterfaces { cause: rtnetlink::Error },
    #[error("Failure while setting interface {interface} to state 'up': {cause}")]
    SetInterfaceUp { interface: Interface, cause: rtnetlink::Error },
    #[error("Failure while joining interface {interface} to bridge {bridge}: {cause}")]
    JoinInterfaceToBridge { interface: Interface, bridge: Interface, cause: rtnetlink::Error },
    #[error("Failure while creating virtual CAN interface '{name}': {cause}")]
    VCanInterfaceCreation { name: NetworkInterfaceName, cause: String},
    #[error("Failure while invoking command line program '{command}': {cause}")]
    CommandLineProgramExecution { command: String, cause: std::io::Error },
    #[error("{message}")]
    Other { message: String },
}
