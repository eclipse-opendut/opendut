use std::fmt::Formatter;
use std::net::Ipv4Addr;
use serde::Serialize;
use crate::peer::executor::ExecutorDescriptor;
use crate::util::net::{NetworkInterfaceDescriptor, NetworkInterfaceName};


#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize)]
pub struct DeviceInterface {
    pub descriptor: NetworkInterfaceDescriptor,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize)]
pub struct EthernetBridge {
    pub name: NetworkInterfaceName,
}

// TODO: move this
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize)]
pub struct GreAddresses {
    pub local_ip: Ipv4Addr,
    pub remote_ip: Ipv4Addr,
}
impl std::fmt::Display for GreAddresses {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}", self.local_ip, self.remote_ip)
    }
}


#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize)]
pub struct GreInterfaces {
    pub address_list: Vec<GreAddresses>,
}

impl std::fmt::Display for GreInterfaces {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let addresses = self.address_list.iter().map(|addr| addr.to_string()).collect::<Vec<_>>().join(", ");
        write!(f, "{}", addresses)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize)]
pub struct Executor {
    pub descriptor: ExecutorDescriptor,
}
