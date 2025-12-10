use leptos::prelude::*;
use opendut_lea_components::{UserInputValue, UserNetworkInterfaceConfiguration};
use opendut_model::util::net::{NetworkInterfaceDescriptor, NetworkInterfaceId, NetworkInterfaceName};
use crate::peers::configurator::types::PeerMisconfigurationError;

#[derive(Clone, Debug)]
pub struct UserPeerNetwork {
    pub network_interfaces: Vec<RwSignal<UserNetworkInterface>>,
    pub bridge_name: UserInputValue,
}

#[derive(Clone, Debug, PartialEq)]
pub struct UserNetworkInterface {
    pub id: NetworkInterfaceId,
    pub name: NetworkInterfaceName,
    pub configuration: UserNetworkInterfaceConfiguration,
}

impl From<NetworkInterfaceDescriptor> for UserNetworkInterface {
    fn from(interface: NetworkInterfaceDescriptor) -> Self {
        Self {
            id: interface.id,
            name: interface.name,
            configuration: UserNetworkInterfaceConfiguration::from(interface.configuration),
        }
    }
}
impl TryFrom<UserNetworkInterface> for NetworkInterfaceDescriptor {
    type Error = PeerMisconfigurationError;

    fn try_from(configuration: UserNetworkInterface) -> Result<Self, Self::Error> {
        Ok(Self {
            id: configuration.id,
            name: configuration.name,
            configuration: configuration.configuration.inner,
        })
    }
}
