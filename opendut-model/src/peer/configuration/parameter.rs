use std::fmt::{Display, Formatter};
use std::net::{IpAddr, Ipv4Addr};
use std::str::FromStr;
use base64::Engine;
use serde::Serialize;
use crate::peer::executor::ExecutorDescriptor;
use crate::peer::PeerId;
use crate::util::net::{NetworkInterfaceDescriptor, NetworkInterfaceName, NetworkInterfaceNameError};
use crate::util::Port;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize)]
pub struct DeviceInterface {
    pub descriptor: NetworkInterfaceDescriptor,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize)]
pub struct EthernetBridge {
    pub name: NetworkInterfaceName,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize)]
pub struct GreInterfaceConfig {
    pub local_ip: Ipv4Addr,
    pub remote_ip: Ipv4Addr,
}

impl GreInterfaceConfig {
    pub fn interface_name(&self) -> Result<NetworkInterfaceName, NetworkInterfaceNameError> {
        let mut addr_bytes = self.local_ip.octets().to_vec();
        addr_bytes.extend(self.remote_ip.octets());
        
        // https://git.kernel.org/pub/scm/network/iproute2/iproute2.git/tree/lib/utils.c?id=1f420318bda3cc62156e89e1b56d60cc744b48ad#n827
        // documents that pretty much anything is allowed except "/", "\0" or whitespace
        // using url safe base64 to avoid forward slash "/" 
        let encoded_addresses = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(addr_bytes);
        let name = format!("gre-{}", encoded_addresses.replace("=", ""));

        NetworkInterfaceName::from_str(&name)
    }
}


#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize)]
pub struct InterfaceJoinConfig {
    pub name: NetworkInterfaceName,
    pub bridge: NetworkInterfaceName,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize)]
pub struct Executor {
    pub descriptor: ExecutorDescriptor,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize)]
pub struct RemotePeerConnectionCheck {
    pub remote_peer_id: PeerId,
    pub remote_ip: IpAddr,
}
impl Display for RemotePeerConnectionCheck {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let Self { remote_peer_id: peer_id, remote_ip } = self;
        write!(f, "{peer_id}: {remote_ip}")
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize)]
pub struct CanConnection {
    pub remote_peer_id: PeerId,
    pub remote_ip: IpAddr,
    pub remote_port: Port,
    pub local_port: Port,
    pub can_interface_name: NetworkInterfaceName,
    /// starts a CAN server for other peers to connect to
    pub local_is_server: bool,
    pub buffer_timeout_microseconds: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize)]
pub struct CanBridge {
    pub name: NetworkInterfaceName,
}

/// Defines a local CAN route between two interfaces.
/// For bidirectional CAN message forwarding, two `CanLocalRoute` entries are needed.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize)]
pub struct CanLocalRoute {
    pub can_source_device_name: NetworkInterfaceName,
    pub can_destination_device_name: NetworkInterfaceName,
}



#[cfg(test)]
mod tests {
    use std::net::Ipv4Addr;
    use std::str::FromStr;
    use crate::peer::configuration::parameter::GreInterfaceConfig;

    #[test_log::test]
    fn test_gre_interface_name() -> anyhow::Result<()> {
        let gre_addresses = GreInterfaceConfig {
            local_ip: Ipv4Addr::from_str("192.168.123.123")?,
            remote_ip: Ipv4Addr::from_str("192.168.123.124")?,
        };
        let name = gre_addresses.interface_name()?;
        assert!(name.name().starts_with("gre-"));
        assert!(name.name().len() < 16);
        
        let illegal_chars = ["=", "/", "+"];
        for illegal_char in illegal_chars.iter() {
            assert!(!name.name().contains(illegal_char));
        }
        let illegal_end_chars = ["-", "_"];
        for illegal_char in illegal_end_chars.iter() {
            assert!(!name.name().ends_with(illegal_char));
        }
        
        Ok(())
    }
}
