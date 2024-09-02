use crate::util::net::NetworkInterfaceName;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct EthernetBridge {
    pub name: NetworkInterfaceName,
}
