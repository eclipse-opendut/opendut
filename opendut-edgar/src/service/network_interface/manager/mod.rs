use std::fmt::Debug;
use std::io;
use std::net::{Ipv4Addr};
use std::sync::Arc;

use futures::TryStreamExt;
use tokio::process::Command;
use tracing::{debug, warn};

use crate::service::network_interface::manager::vcan::Vcan;
use gretap::Gretap;
use interface::Interface;
use opendut_model::util::net::NetworkInterfaceName;
use crate::service::network_interface::manager::can::{BitTiming, CanFD, CanInterfaceConfiguration};

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
            .map_err(|source| Error::Connecting { source })?;
        tokio::spawn(connection);

        Ok(Arc::new(Self { handle }))
    }

    pub async fn list_interfaces(&self) -> Result<Vec<Interface>, Error> {
        let interfaces = self.handle
            .link()
            .get()
            .execute()
            .try_collect::<Vec<_>>().await
            .map_err(|source| Error::ListInterfaces { source: source.into() })?
            .into_iter()
            .filter_map(|link_message| {
                let index = link_message.header.index;
                Interface::try_from(link_message)
                    .inspect_err(|source| warn!("Could not determine attributes of interface with index '{index}': {source}"))
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
            .map_err(|source| Error::BridgeCreation { name: name.clone(), source: source.into() })?;

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
            .map_err(|source| Error::GretapCreation { name: name.clone(), source: source.into() })?;
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
            .map_err(|source| Error::SetInterfaceUp { interface: Box::new(interface.clone()), source: source.into() })?;
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
            .map_err(|source| Error::SetInterfaceDown { interface: Box::new(interface.clone()), source: source.into() })?;
        Ok(())
    }

    pub async fn update_can_interface(&self, interface_name: &NetworkInterfaceName, can_config: &CanInterfaceConfiguration) -> Result<(), Error> {
        let CanInterfaceConfiguration { bit_timing, fd } = can_config;
        let BitTiming { bitrate, sample_point } = bit_timing;

        debug!("Update CAN interface {interface_name} with bitrate: {bitrate}, sample-point: {sample_point}");

        let mut ip_link_command = Command::new("ip");
        ip_link_command.arg("link")
            .arg("set")
            .arg(interface_name.name())
            .arg("type")
            .arg("can")
            .arg("bitrate")
            .arg(bitrate.to_string())
            .arg("sample-point")
            .arg(sample_point.to_string());

        if let CanFD::Enabled(BitTiming { bitrate: data_bitrate, sample_point: data_sample_point }) = fd {
            debug!("Update CAN interface {interface_name} with fd: 'enabled', data_bitrate: {data_bitrate}, data_sample_point: {data_sample_point}");

            ip_link_command
                .arg("dbitrate")
                .arg(data_bitrate.to_string())
                .arg("dsample-point")
                .arg(data_sample_point.to_string())
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
            .map_err(|source| Error::CommandLineProgramExecution { command: format!("{ip_link_command:?}"), source })?;

        if !output.status.success() {
            return Err(Error::CanInterfaceUpdate { name: interface_name.clone(), source: format!("{:?}", String::from_utf8_lossy(&output.stderr).trim()) });
        }

        Ok(())
    }

    pub async fn delete_interface(&self, interface: &Interface) -> Result<(), Error> {
        self.handle
            .link()
            .del(interface.index)
            .execute().await
            .map_err(|source| Error::DeleteInterface { interface: Box::new(interface.clone()), source: source.into() })?;
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
            .map_err(|error| Error::VcanInterfaceCreation { name: name.clone(), source: error.to_string() })?;
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
            .map_err(|error| Error::ModificationFailure { name: name.clone(), source: error.to_string() })?;

        let interface = self.try_find_interface(name).await?;
        Ok(interface)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failure while creating bridge '{name}': {source}")]
    BridgeCreation { name: NetworkInterfaceName, source: Box<rtnetlink::Error> },
    #[error("Failed to establish connection to netlink: {source}")]
    Connecting { source: io::Error },
    #[error("Failure while deleting interface {interface}: {source}")]
    DeleteInterface { interface: Box<Interface>, source: Box<rtnetlink::Error> },
    #[error("Failure while creating gretap interface '{name}': {source}")]
    GretapCreation { name: NetworkInterfaceName, source: Box<rtnetlink::Error> },
    #[error("Interface with name '{name}' not found.")]
    InterfaceNotFound { name: NetworkInterfaceName },
    #[error("Failure while listing interfaces: {source}")]
    ListInterfaces { source: Box<rtnetlink::Error> },
    #[error("Failure while setting interface {interface} to state 'up': {source}")]
    SetInterfaceUp { interface: Box<Interface>, source: Box<rtnetlink::Error> },
    #[error("Failure while setting interface {interface} to state 'down': {source}")]
    SetInterfaceDown { interface: Box<Interface>, source: Box<rtnetlink::Error> },
    #[error("Failure while joining interface {interface} to bridge {bridge}: {source}")]
    JoinInterfaceToBridge { interface: Box<Interface>, bridge: Box<Interface>, source: Box<rtnetlink::Error> },
    #[error("Failure while creating virtual CAN interface '{name}': {message}")]
    VcanInterfaceCreation { name: NetworkInterfaceName, message: String },
    #[error("Failed to modify interface '{name}': {message}")]
    ModificationFailure { name: NetworkInterfaceName, message: String},
    #[error("Failure during updating CAN interface '{name}': {message}")]
    CanInterfaceUpdate { name: NetworkInterfaceName, message: String},
    #[error("Failure while invoking command line program '{command}': {source}")]
    CommandLineProgramExecution { command: String, source: std::io::Error },
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
