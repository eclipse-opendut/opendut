use crate::util::net::{NetworkInterfaceDescriptor, NetworkInterfaceName};


#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DeviceInterface {
    pub descriptor: NetworkInterfaceDescriptor,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct EthernetBridge {
    pub name: NetworkInterfaceName,
}
