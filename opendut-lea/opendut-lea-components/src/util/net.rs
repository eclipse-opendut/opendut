use opendut_model::util::net::NetworkInterfaceConfiguration;

#[derive(Clone, Debug, PartialEq)]
pub struct UserNetworkInterfaceConfiguration {
    pub inner: NetworkInterfaceConfiguration
}

impl UserNetworkInterfaceConfiguration {
    pub fn display_name(&self) -> String {
        match self.inner {
            NetworkInterfaceConfiguration::Ethernet => String::from("Ethernet"),
            NetworkInterfaceConfiguration::Can { .. } => String::from("CAN")
        }
    }
}

impl From<NetworkInterfaceConfiguration> for UserNetworkInterfaceConfiguration {
    fn from(value: NetworkInterfaceConfiguration) -> Self {
        Self { inner: value }
    }
}
