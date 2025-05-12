use std::fmt::Formatter;
use std::net::Ipv4Addr;
use std::str::FromStr;
use base64::Engine;
use serde::Serialize;
use crate::peer::executor::ExecutorDescriptor;
use crate::util::net::{NetworkInterfaceDescriptor, NetworkInterfaceName, NetworkInterfaceNameError};


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
impl std::fmt::Display for GreInterfaceConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}", self.local_ip, self.remote_ip)
    }
}

impl GreInterfaceConfig {
    pub fn interface_name(&self) -> Result<NetworkInterfaceName, NetworkInterfaceNameError> {
        let mut addr_bytes = self.local_ip.octets().to_vec();
        addr_bytes.extend(self.remote_ip.octets());
        
        let encoded_addresses = base64::engine::general_purpose::STANDARD.encode(addr_bytes);
        let name = format!("gre-{}", encoded_addresses.replace("=", ""));

        NetworkInterfaceName::from_str(&name)
    }
}


#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize)]
pub struct InterfaceJoinConfig {
    pub name: NetworkInterfaceName,
    pub bridge: NetworkInterfaceName,
}

impl std::fmt::Display for InterfaceJoinConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}", self.name.name(), self.bridge.name())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize)]
pub struct Executor {
    pub descriptor: ExecutorDescriptor,
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
        
        Ok(())
    }
}