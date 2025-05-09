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

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize)]
pub struct Executor {
    pub descriptor: ExecutorDescriptor,
}
