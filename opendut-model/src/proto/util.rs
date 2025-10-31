use crate::proto::util::ip_address::Address;
use crate::util;
use crate::util::net::NetworkInterfaceConfiguration;
use pem::Pem;
use opendut_util::conversion;
use opendut_util::proto::ConversionResult;

opendut_util::include_proto!("opendut.model.util");

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


conversion! {
    type Model = util::Port;
    type Proto = Port;

    fn from(value: Model) -> Proto {
        Proto { value: value.0 as u32 }
    }

    fn try_from(value: Proto) -> ConversionResult<Model> {
        let port: u16 = value.value
            .try_into()
            .map_err(|_| ErrorBuilder::message("Port value is out of range"))?;

        Ok(util::Port(port))
    }
}

conversion! {
    type Model = url::Url;
    type Proto = Url;

    fn from(value: Model) -> Proto {
        Proto { value: value.to_string() }
    }

    fn try_from(value: Proto) -> ConversionResult<Model> {
        url::Url::parse(&value.value)
            .map_err(|cause| ErrorBuilder::message(format!("Url could not be parsed: {cause}")))
    }
}

conversion! {
    type Model = std::net::IpAddr;
    type Proto = IpAddress;

    fn from(value: Model) -> Proto {
        match value {
            std::net::IpAddr::V4(address) => Proto {
                address: Some(ip_address::Address::IpV4(IpV4Address::from(address))),
            },
            std::net::IpAddr::V6(address) => Proto {
                address: Some(ip_address::Address::IpV6(IpV6Address::from(address))),
            },
        }
    }

    fn try_from(value: Proto) -> ConversionResult<Model> {
        let address = extract!(value.address)?;

        let address = match address {
            Address::IpV4(address) => std::net::IpAddr::V4(
                std::net::Ipv4Addr::try_from(address)
                    .map_err(|cause| ErrorBuilder::message(cause.to_string()))?
            ),
            Address::IpV6(address) => std::net::IpAddr::V6(
                std::net::Ipv6Addr::try_from(address)
                    .map_err(|cause| ErrorBuilder::message(cause.to_string()))?
            ),
        };
        Ok(address)
    }
}

conversion! {
    type Model = std::net::Ipv4Addr;
    type Proto = IpV4Address;

    fn from(value: Model) -> Proto {
        Proto {
            value: Vec::from(value.octets()),
        }
    }

    fn try_from(value: Proto) -> ConversionResult<Model> {
        const IPV4_LENGTH: usize = 4; //bytes

        let octets: [u8; IPV4_LENGTH] = value.value[0..IPV4_LENGTH].try_into()
            .map_err(|cause| ErrorBuilder::message(format!("IPv4 address could not be parsed, because it did not have the correct length ({IPV4_LENGTH} bytes): {cause}")))?;

        Ok(std::net::Ipv4Addr::from(octets))
    }
}

conversion! {
    type Model = std::net::Ipv6Addr;
    type Proto = IpV6Address;

    fn from(value: Model) -> Proto {
        Proto {
            value: Vec::from(value.octets()),
        }
    }

    fn try_from(value: Proto) -> ConversionResult<Model> {
        const IPV6_LENGTH: usize = 16; //bytes

        let octets: [u8; IPV6_LENGTH] = value.value[0..IPV6_LENGTH].try_into()
            .map_err(|cause| ErrorBuilder::message(format!("IPv6 address could not be parsed, because it did not have the correct length ({IPV6_LENGTH} bytes): {cause}")))?;

        Ok(std::net::Ipv6Addr::from(octets))
    }
}

conversion! {
    type Model = crate::util::net::NetworkInterfaceName;
    type Proto = NetworkInterfaceName;

    fn from(value: Model) -> Proto {
        Proto {
            name: value.name(),
        }
    }

    fn try_from(value: Proto) -> ConversionResult<Model> {
        Model::try_from(value.name)
            .map_err(|cause| ErrorBuilder::message(format!("Failed to parse InterfaceName from proto: {cause}")))
    }
}

conversion! {
    type Model = crate::util::net::Certificate;
    type Proto = Certificate;

    fn from(value: Model) -> Proto {
        Proto {
            tag: value.0.tag().to_owned(),
            content: Vec::from(value.0.contents()),
        }
    }

    fn try_from(value: Proto) -> ConversionResult<Model> {
        Ok(util::net::Certificate(Pem::new(value.tag, value.content)))
    }
}

conversion! {
    type Model = crate::util::net::NetworkInterfaceDescriptor;
    type Proto = NetworkInterfaceDescriptor;

    fn from(value: Model) -> Proto {
        let config = match value.configuration {
            NetworkInterfaceConfiguration::Ethernet => network_interface_descriptor::Configuration::Ethernet(EthernetInterfaceConfiguration {}),
            NetworkInterfaceConfiguration::Can { 
                bitrate, 
                sample_point, 
                fd: flexible_data_rate, 
                data_bitrate, 
                data_sample_point
            } => network_interface_descriptor::Configuration::Can(
                CanInterfaceConfiguration {
                    bitrate,
                    sample_point: sample_point.sample_point_times_1000(),
                    flexible_data_rate,
                    data_bitrate,
                    data_sample_point: data_sample_point.sample_point_times_1000()
                }
            ),
        };

        Proto {
            id: Some(value.id.into()),
            name: Some(value.name.into()),
            configuration: Some(config),
        }
    }

    fn try_from(value: Proto) -> ConversionResult<Model> {
        let id = extract!(value.id)?.try_into()?;

        let name = extract!(value.name)?.try_into()?;

        let configuration = match extract!(value.configuration)? {
                network_interface_descriptor::Configuration::Ethernet(_) => NetworkInterfaceConfiguration::Ethernet,
                network_interface_descriptor::Configuration::Can(can_config) => NetworkInterfaceConfiguration::Can { 
                    bitrate: can_config.bitrate, 
                    sample_point: can_config.sample_point.try_into()
                        .map_err(|cause| ErrorBuilder::message(format!("Sample point could not be converted: {cause}")))?,
                    fd: can_config.flexible_data_rate, 
                    data_bitrate: can_config.data_bitrate, 
                    data_sample_point: can_config.data_sample_point.try_into()
                        .map_err(|cause| ErrorBuilder::message(format!("Sample point could not be converted: {cause}")))?,
                },
            };

        Ok(Model {
            id,
            name,
            configuration,
        })
    }
}

conversion! {
    type Model = crate::util::net::NetworkInterfaceId;
    type Proto = NetworkInterfaceId;

    fn from(value: Model) -> Proto {
        Proto {
            uuid: Some(value.uuid.into()),
        }
    }

    fn try_from(value: Proto) -> ConversionResult<Model> {
        extract!(value.uuid)
            .map(|uuid| Model { uuid: uuid.into() })
    }
}

conversion! {
    type Model = crate::util::net::ClientSecret;
    type Proto = ClientSecret;

    fn from(value: Model) -> Proto {
        Proto {
            value: value.0
        }
    }

    fn try_from(value: Proto) -> ConversionResult<Model> {
        Model::try_from(value.value)
            .map_err(|cause| ErrorBuilder::message(cause.to_string()))
    }
}

conversion! {
    type Model = crate::util::net::ClientId;
    type Proto = ClientId;

    fn from(value: Model) -> Proto {
        Proto {
            value: value.0
        }
    }

    fn try_from(value: Proto) -> ConversionResult<Model> {
        Model::try_from(value.value)
            .map_err(|cause| ErrorBuilder::message(cause.to_string()))
    }
}

conversion! {
    type Model = crate::util::net::OAuthScope;
    type Proto = OAuthScope;

    fn from(value: Model) -> Proto {
        Proto {
            value: value.0
        }
    }

    fn try_from(value: Proto) -> ConversionResult<Model> {
        Model::try_from(value.value)
            .map_err(|cause| ErrorBuilder::message(cause.to_string()))
    }
}

conversion! {
    type Model = crate::util::net::AuthConfig;
    type Proto = AuthConfig;

    fn from(value: Model) -> Proto {
        let config = match value {
            Model::Disabled => { auth_config::Config::Disabled( AuthConfigDisabled {}) }
            Model::Enabled {
                issuer_url,
                client_id,
                client_secret,
                scopes
            } => auth_config::Config::Enabled ( AuthConfigEnabled {
                issuer_url: Some(issuer_url.into()),
                client_id: Some(client_id.into()),
                client_secret: Some(client_secret.into()),
                scopes: scopes.into_iter().map(|scope| scope.into()).collect()
            })
        };

        Proto {
            config: Some(config),
        }
    }

    fn try_from(value: Proto) -> ConversionResult<Model> {
        let config = match extract!(value.config)? {
            auth_config::Config::Disabled(_) => Model::Disabled,
            auth_config::Config::Enabled(auth_config) => {
                let issuer_url = extract!(auth_config.issuer_url)
                    .and_then(|url| url::Url::parse(&url.value)
                        .map_err(|cause| ErrorBuilder::message(format!("Authorization Provider Issuer URL could not be parsed: {cause}")))
                    )?;

                let client_id: crate::util::net::ClientId = extract!(auth_config.client_id)?.try_into()?;

                let client_secret: crate::util::net::ClientSecret = extract!(auth_config.client_secret)?.try_into()?;

                let scopes: Vec<crate::util::net::OAuthScope> = auth_config.scopes
                    .into_iter()
                    .map(TryFrom::try_from)
                    .collect::<Result<_, _>>()?;

                Model::Enabled {
                    issuer_url,
                    client_id,
                    client_secret,
                    scopes,
                }
            }
        };

        Ok(config)
    }
}
