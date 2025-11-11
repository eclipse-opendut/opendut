use std::fmt::Debug;
use std::io;
use std::net::{Ipv4Addr};
use std::sync::Arc;

use futures::TryStreamExt;
use tokio::process::Command;
use tracing::{debug, error, warn};

use crate::service::network_interface::manager::vcan::VCan;
use gretap::Gretap;
use interface::Interface;
use opendut_model::util::net::{NetworkInterfaceConfiguration, NetworkInterfaceDescriptor, NetworkInterfaceName};

mod gretap;
mod list_joined_interfaces;
pub mod interface;
pub mod vcan;

pub mod bridge;
pub mod altname;
pub mod can;

pub type NetworkInterfaceManagerRef = Arc<NetworkInterfaceManager>;

pub struct NetworkInterfaceManager {
    pub(crate) handle: rtnetlink::Handle,
}
impl NetworkInterfaceManager {
    pub fn create() -> Result<NetworkInterfaceManagerRef, Error> {
        let (connection, handle, _) = rtnetlink::new_connection()
            .map_err(|cause| Error::Connecting { cause })?;
        tokio::spawn(connection);

        Ok(Arc::new(Self { handle }))
    }

    pub async fn list_interfaces(&self) -> Result<Vec<Interface>, Error> {
        let interfaces = self.handle
            .link()
            .get()
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
            .add(
                rtnetlink::LinkBridge::new(&name.name())
                    .build()
            )
            .execute().await
            .map_err(|cause| Error::BridgeCreation { name: name.clone(), cause: cause.into() })?;

        let interface = self.try_find_interface(name).await?;
        Ok(interface)
    }

    // We only support IPv4 for now, as NetBird only assigns IPv4 addresses to peers.
    // This does not prevent IPv6 traffic from being routed between peers.
    pub async fn create_gretap_v4_interface(&self, name: &NetworkInterfaceName, local_ip: &Ipv4Addr, remote_ip: &Ipv4Addr) -> Result<Interface, Error> {
        self.handle
            .link()
            .add(
                rtnetlink::LinkUnspec::new_with_name(&name.name())
                    .gretap_v4(local_ip, remote_ip)
                    .build()
            )
            .execute().await
            .map_err(|cause| Error::GretapCreation { name: name.clone(), cause: cause.into() })?;
        let interface = self.try_find_interface(name).await?;
        Ok(interface)
    }

    pub async fn set_interface_up(&self, interface: &Interface) -> Result<(), Error> {
        debug!("Set interface {} up.", interface.name);
        self.handle
            .link()
            .set(
                rtnetlink::LinkUnspec::new_with_index(interface.index)
                    .up()
                    .build()
            )
            .execute().await
            .map_err(|cause| Error::SetInterfaceUp { interface: Box::new(interface.clone()), cause: cause.into() })?;
        Ok(())
    }

    pub async fn set_interface_down(&self, interface: &Interface) -> Result<(), Error> {
        debug!("Set interface {} down.", interface.name);
        self.handle
            .link()
            .set(
                rtnetlink::LinkUnspec::new_with_index(interface.index)
                    .down()
                    .build()
            )
            .execute().await
            .map_err(|cause| Error::SetInterfaceDown { interface: Box::new(interface.clone()), cause: cause.into() })?;
        Ok(())
    }

    pub async fn update_interface(&self, network_interface_descriptor: NetworkInterfaceDescriptor) -> Result<(), Error> {
        match network_interface_descriptor.configuration {
            NetworkInterfaceConfiguration::Can { bitrate, sample_point, fd, data_bitrate, data_sample_point } => {
                debug!("Update CAN interface {interface} with bitrate: {bitrate}, sample-point: {sample_point}, fd: {fd}, data_bitrate: {data_bitrate}, data_sample_point: {data_sample_point}", interface=network_interface_descriptor.name);

                let mut ip_link_command = Command::new("ip");
                ip_link_command.arg("link")
                    .arg("set")
                    .arg(network_interface_descriptor.name.name())
                    .arg("type")
                    .arg("can")
                    .arg("bitrate")
                    .arg(bitrate.to_string())
                    .arg("sample-point")
                    .arg(sample_point.sample_point().to_string());

                if fd {
                    ip_link_command
                        .arg("dbitrate")
                        .arg(data_bitrate.to_string())
                        .arg("dsample-point")
                        .arg(data_sample_point.sample_point().to_string())
                        .arg("fd")
                        .arg("on");
                } else {
                    ip_link_command
                        .arg("fd")
                        .arg("off");
                }

                let output = ip_link_command
                    .output()
                    .await
                    .map_err(|cause| Error::CommandLineProgramExecution { command: format!("{ip_link_command:?}"), cause })?;

                if !output.status.success() {
                    return Err(Error::CanInterfaceUpdate { name: network_interface_descriptor.name.clone(), cause: format!("{:?}", String::from_utf8_lossy(&output.stderr).trim()) });
                }
            }
            NetworkInterfaceConfiguration::Vcan => {
                debug!("Update VCAN interface {interface}...", interface=network_interface_descriptor.name);

                let mut ip_link_command = Command::new("ip");
                ip_link_command.arg("link")
                    .arg("set")
                    .arg(network_interface_descriptor.name.name())
                    .arg("type")
                    .arg("vcan");

                let output = ip_link_command
                    .output()
                    .await
                    .map_err(|cause| Error::CommandLineProgramExecution { command: format!("{ip_link_command:?}"), cause })?;

                if !output.status.success() {
                    return Err(Error::CanInterfaceUpdate { name: network_interface_descriptor.name.clone(), cause: format!("{:?}", String::from_utf8_lossy(&output.stderr).trim()) });
                }
            }
            NetworkInterfaceConfiguration::Ethernet => {} //do nothing
        }

        Ok(())
    }

    pub async fn delete_interface(&self, interface: &Interface) -> Result<(), Error> {
        self.handle
            .link()
            .del(interface.index)
            .execute().await
            .map_err(|cause| Error::DeleteInterface { interface: Box::new(interface.clone()), cause: cause.into() })?;
        Ok(())
    }

    pub async fn create_vcan_interface(&self, name: &NetworkInterfaceName) -> Result<Interface, Error> {
        self.handle
            .link()
            .add(
                rtnetlink::LinkUnspec::new_with_name(&name.name())
                    .vcan()
                    .build()
            )
            .execute()
            .await
            .map_err(|error| Error::VCanInterfaceCreation { name: name.clone(), cause: error.to_string() })?;
        let interface = self.try_find_interface(name).await?;
        Ok(interface)
    }

    #[allow(unused)]
    pub async fn create_dummy_ipv4_interface(&self, name: &NetworkInterfaceName) -> Result<Interface, Error> {
        self.handle
            .link()
            .add(
                rtnetlink::LinkDummy::new(&name.name())
                    .build()
            )
            .execute()
            .await
            .map_err(|error| Error::ModificationFailure { name: name.clone(), cause: error.to_string() })?;

        let interface = self.try_find_interface(name).await?;
        Ok(interface)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failure while creating bridge '{name}': {cause}")]
    BridgeCreation { name: NetworkInterfaceName, cause: Box<rtnetlink::Error> },
    #[error("Failed to establish connection to netlink: {cause}")]
    Connecting { cause: io::Error },
    #[error("Failure while deleting interface {interface}: {cause}")]
    DeleteInterface { interface: Box<Interface>, cause: Box<rtnetlink::Error> },
    #[error("Failure while creating gretap interface '{name}': {cause}")]
    GretapCreation { name: NetworkInterfaceName, cause: Box<rtnetlink::Error> },
    #[error("Interface with name '{name}' not found.")]
    InterfaceNotFound { name: NetworkInterfaceName },
    #[error("Failure while listing interfaces: {cause}")]
    ListInterfaces { cause: Box<rtnetlink::Error> },
    #[error("Failure while setting interface {interface} to state 'up': {cause}")]
    SetInterfaceUp { interface: Box<Interface>, cause: Box<rtnetlink::Error> },
    #[error("Failure while setting interface {interface} to state 'down': {cause}")]
    SetInterfaceDown { interface: Box<Interface>, cause: Box<rtnetlink::Error> },
    #[error("Failure while joining interface {interface} to bridge {bridge}: {cause}")]
    JoinInterfaceToBridge { interface: Box<Interface>, bridge: Box<Interface>, cause: Box<rtnetlink::Error> },
    #[error("Failure while creating virtual CAN interface '{name}': {cause}")]
    VCanInterfaceCreation { name: NetworkInterfaceName, cause: String },
    #[error("Failed to modify interface '{name}': {cause}")]
    ModificationFailure { name: NetworkInterfaceName, cause: String},
    #[error("Failure during updating CAN interface '{name}': {cause}")]
    CanInterfaceUpdate { name: NetworkInterfaceName, cause: String},
    #[error("Failure while invoking command line program '{command}': {cause}")]
    CommandLineProgramExecution { command: String, cause: std::io::Error },
}


#[cfg(test)]
mod tests {
    use crate::service::network_interface::manager::NetworkInterfaceManager;
    use tracing::debug;

    /// How to run integration tests in dev environment: 
    /// cargo ci integration-test

    #[test_with::env(RUN_EDGAR_NETLINK_INTEGRATION_TESTS)]
    #[test_log::test(tokio::test)]
    async fn test_list_interfaces() -> anyhow::Result<()> {
        let (connection, handle, _) = rtnetlink::new_connection()?;
        tokio::spawn(connection);

        let manager = NetworkInterfaceManager { handle };
        let result = manager.list_interfaces().await?;
        assert!(!result.is_empty());

        debug!("Network interfaces: {:?}", result);
        Ok(())
    }
}
