use crate::proto::{ConversionError, ConversionErrorBuilder};
use crate::proto::util::ip_address::Address;
use crate::util;

include!(concat!(env!("OUT_DIR"), "/opendut.types.util.rs"));

impl From<uuid::Uuid> for Uuid {
    fn from(value: uuid::Uuid) -> Self {
        let (msb, lsb) = value.as_u64_pair();
        Self { msb, lsb }
    }
}

impl From<Uuid> for uuid::Uuid {
    fn from(value: Uuid) -> Self {
        Self::from_u64_pair(value.msb, value.lsb)
    }
}

impl From<String> for Hostname {
    fn from(value: String) -> Self {
        Self { value }
    }
}

impl From<Hostname> for String {
    fn from(value: Hostname) -> Self {
        value.value
    }
}

impl From<util::Hostname> for Hostname {
    fn from(value: util::Hostname) -> Self {
        Self { value: value.0 }
    }
}

impl From<Hostname> for util::Hostname {
    fn from(value: Hostname) -> Self {
        util::Hostname(value.value)
    }
}

impl From<u16> for Port {
    fn from(value: u16) -> Self {
        Self { value: value as u32 }
    }
}

impl TryFrom<Port> for u16 {
    type Error = ConversionError;

    fn try_from(value: Port) -> Result<Self, Self::Error> {
        type ErrorBuilder = ConversionErrorBuilder<Port, u16>;

        value.value
            .try_into()
            .map_err(|_| ErrorBuilder::new("Port value is out of range"))
    }
}

impl From<util::Port> for Port {
    fn from(value: util::Port) -> Self {
        Self { value: value.0 as u32 }
    }
}

impl TryFrom<Port> for util::Port {
    type Error = ConversionError;

    fn try_from(value: Port) -> Result<Self, Self::Error> {
        type ErrorBuilder = ConversionErrorBuilder<Port, u16>;

        let port: u16 = value.value
            .try_into()
            .map_err(|_| ErrorBuilder::new("Port value is out of range"))?;

        Ok(util::Port(port))
    }
}

impl From<url::Url> for Url {
    fn from(value: url::Url) -> Self {
        Self { value: value.to_string() }
    }
}

impl TryFrom<Url> for url::Url {
    type Error = ConversionError;

    fn try_from(value: Url) -> Result<Self, Self::Error> {
        type ErrorBuilder = ConversionErrorBuilder<Url, url::Url>;

        url::Url::parse(&value.value)
            .map_err(|cause| ErrorBuilder::new(format!("Url could not be parsed: {}", cause)))
    }
}

impl From<std::net::IpAddr> for IpAddress {
    fn from(value: std::net::IpAddr) -> Self {
        match value {
            std::net::IpAddr::V4(address) => Self {
                address: Some(ip_address::Address::IpV4(IpV4Address::from(address))),
            },
            std::net::IpAddr::V6(address) => Self {
                address: Some(ip_address::Address::IpV6(IpV6Address::from(address))),
            },
        }
    }
}
impl TryFrom<IpAddress> for std::net::IpAddr {
    type Error = ConversionError;

    fn try_from(value: IpAddress) -> Result<Self, Self::Error> {
        type ErrorBuilder = ConversionErrorBuilder<IpAddress, std::net::IpAddr>;

        let address = value.address
            .ok_or(ErrorBuilder::new("IP address not set"))?;

        let address = match address {
            Address::IpV4(address) => std::net::IpAddr::V4(
                std::net::Ipv4Addr::try_from(address)
                    .map_err(|cause| ErrorBuilder::new(cause.to_string()))?
            ),
            Address::IpV6(address) => std::net::IpAddr::V6(
                std::net::Ipv6Addr::try_from(address)
                    .map_err(|cause| ErrorBuilder::new(cause.to_string()))?
            ),
        };
        Ok(address)
    }
}

impl From<std::net::Ipv4Addr> for IpV4Address {
    fn from(value: std::net::Ipv4Addr) -> Self {
        Self {
            value: Vec::from(value.octets()),
        }
    }
}
impl TryFrom<IpV4Address> for std::net::Ipv4Addr {
    type Error = ConversionError;

    fn try_from(value: IpV4Address) -> Result<Self, Self::Error> {
        type ErrorBuilder = ConversionErrorBuilder<IpV4Address, std::net::Ipv4Addr>;

        const IPV4_LENGTH: usize = 4; //bytes

        let octets: [u8; IPV4_LENGTH] = value.value[0..IPV4_LENGTH].try_into()
            .map_err(|cause| ErrorBuilder::new(format!("IPv4 address could not be parsed, because it did not have the correct length ({IPV4_LENGTH} bytes): {}", cause)))?;

        Ok(std::net::Ipv4Addr::from(octets))
    }
}

impl From<std::net::Ipv6Addr> for IpV6Address {
    fn from(value: std::net::Ipv6Addr) -> Self {
        Self {
            value: Vec::from(value.octets()),
        }
    }
}
impl TryFrom<IpV6Address> for std::net::Ipv6Addr {
    type Error = ConversionError;

    fn try_from(value: IpV6Address) -> Result<Self, Self::Error> {
        type ErrorBuilder = ConversionErrorBuilder<IpV6Address, std::net::Ipv6Addr>;

        const IPV6_LENGTH: usize = 16; //bytes

        let octets: [u8; IPV6_LENGTH] = value.value[0..IPV6_LENGTH].try_into()
            .map_err(|cause| ErrorBuilder::new(format!("IPv6 address could not be parsed, because it did not have the correct length ({IPV6_LENGTH} bytes): {}", cause)))?;

        Ok(std::net::Ipv6Addr::from(octets))
    }
}

impl From<crate::util::net::NetworkInterfaceName> for NetworkInterfaceName {
    fn from(value: crate::util::net::NetworkInterfaceName) -> Self {
        Self {
            name: value.name(),
        }
    }
}
impl TryFrom<NetworkInterfaceName> for crate::util::net::NetworkInterfaceName {
    type Error = ConversionError;

    fn try_from(value: NetworkInterfaceName) -> Result<Self, Self::Error> {
        type ErrorBuilder = ConversionErrorBuilder<NetworkInterfaceName, crate::util::net::NetworkInterfaceName>;

        crate::util::net::NetworkInterfaceName::try_from(value.name)
            .map_err(|cause| ErrorBuilder::new(format!("Failed to parse InterfaceName from proto: {cause}")))
    }
}
