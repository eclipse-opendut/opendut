use crate::service::network_interface::manager::vcan::VIRTUAL_CAN_INTERFACE_TYPE;
use rtnetlink::packet_route::link::{InfoData, InfoGreTap, InfoKind, LinkAttribute, LinkFlags, LinkInfo, LinkMessage, Prop};
use rtnetlink::packet_core::Nla;
use opendut_model::util::net::{NetworkInterfaceName, NetworkInterfaceNameError};
use std::fmt::Formatter;
use std::net::Ipv4Addr;


pub const CAN_INTERFACE_TYPE: &str = "can";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Interface {
    pub index: u32,
    pub name: NetworkInterfaceName,
    /// interface joined to a bridge with given index
    pub controller_index: Option<u32>,
    pub kind: NetlinkInterfaceKind,
    pub address: Option<Ipv4Addr>,
    pub alternative_names: Vec<String>,
    pub alias: Option<String>,
    pub link_flags: LinkFlags,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NetlinkInterfaceKind {
    Bridge,
    GreTap {
        local: Ipv4Addr,
        remote: Ipv4Addr,
    },
    VCan,
    Can(Option<InfoData>),
    Other(InfoKind),
}

impl std::fmt::Display for Interface {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}: {}]", self.index, self.name)
    }
}


#[derive(thiserror::Error, Debug)]
pub enum NetlinkConversionError {
    #[error("Could not find name attribute!")]
    NameAttributeNotFound,
    #[error("Could not parse interface name!")]
    NetworkInterfaceName(#[from] NetworkInterfaceNameError),
    #[error("Could not determine attributes of GRE interface!")]
    NetworkInterfaceGre,
}

impl TryFrom<LinkMessage> for Interface {
    type Error = NetlinkConversionError;

    fn try_from(link_message: LinkMessage) -> Result<Self, Self::Error> {
        let index = link_message.header.index;
        let interface_name = link_message.attributes.iter()
            .find_map(|nla| match nla {
                LinkAttribute::IfName(name) => Some(name),
                _ => None,
            })
            .cloned()
            .ok_or(NetlinkConversionError::NameAttributeNotFound)?;
        let name = NetworkInterfaceName::try_from(interface_name.clone())
            .map_err(NetlinkConversionError::NetworkInterfaceName)?;
        let interface_kind = link_message.attributes.iter()
            .find_map(|link_attribute| {
                if let LinkAttribute::LinkInfo(link_info) = link_attribute {
                    link_info.iter().find_map(|link_info| {
                        if let LinkInfo::Kind(kind) = link_info {
                            Some(kind)
                        } else {
                            None
                        }
                    })
                } else {
                    None
                }
            })
            .cloned()
            .unwrap_or(InfoKind::Other("unknown".to_string()));

        let controller_index = find_controller_index(&link_message.attributes);
        let address = find_address_attribute(&link_message.attributes);
        let alternative_names = find_alternative_names(&link_message.attributes);
        let alias = find_interface_alias(&link_message.attributes);
        
        // ip -json -details link show dev <NAME> | jq .[0].linkinfo.info_data
        let interface_info_data = link_message.attributes.iter().find_map(|link_attribute| {
            if let LinkAttribute::LinkInfo(link_info) = link_attribute {
                link_info.iter().find_map(|link_info| {
                    if let LinkInfo::Data(kind) = link_info {
                        Some(kind.clone())
                    } else {
                        None
                    }
                })
            } else {
                None
            }
        });

        let kind = match interface_kind {
            InfoKind::Bridge => { NetlinkInterfaceKind::Bridge }
            InfoKind::GreTap => { determine_gre_interface(&link_message)? }
            InfoKind::Other(ref name) => {
                if name.eq(VIRTUAL_CAN_INTERFACE_TYPE) {
                    NetlinkInterfaceKind::VCan
                } else if name.eq(&CAN_INTERFACE_TYPE) {
                    NetlinkInterfaceKind::Can(interface_info_data)                
                } else {
                    NetlinkInterfaceKind::Other(interface_kind)
                }
            }
            _ => { NetlinkInterfaceKind::Other(interface_kind) }
        };
        let link_flag = link_message.header.flags;

        Ok(Self {
            index,
            name,
            controller_index,
            kind,
            address,
            alternative_names,
            alias,
            link_flags: link_flag,
        })
    }
}

fn find_controller_index(attributes: &[LinkAttribute]) -> Option<u32> {
    attributes.iter().find_map(|link_attribute| {
        if let LinkAttribute::Controller(index) = link_attribute {
            Some(*index)
        } else {
            None
        }
    })
}

fn find_address_attribute(attributes: &[LinkAttribute]) -> Option<Ipv4Addr> {
    attributes.iter().find_map(|link_attribute| {
        if let LinkAttribute::Address(address_bytes) = link_attribute {
            let buffer: &mut [u8; 4] = &mut [0; 4];
            buffer.clone_from_slice(&address_bytes[0..4]);
            let octets: [u8; 4] = buffer[0..4].try_into().unwrap();
            let result = Ipv4Addr::from(octets);
            Some(result)
        } else {
            None
        }
    })
}

fn find_alternative_names(attributes: &[LinkAttribute]) -> Vec<String> {
    attributes.iter().find_map(|link_attribute| {
        if let LinkAttribute::PropList(properties) = link_attribute {
            let alt_name_list = properties.iter().filter_map(|property| {
                match property {
                    Prop::AltIfName(name) => {
                        Some(name.to_string())
                    }
                    _ => {
                        None
                    }
                }
            }).collect::<Vec<String>>();
            Some(alt_name_list)
        } else {
            
            None
        }
    }).unwrap_or_default()
}

fn find_interface_alias(attributes: &[LinkAttribute]) -> Option<String> {
    attributes.iter().find_map(|link_attribute| {
        if let LinkAttribute::IfAlias(alias) = link_attribute {
            Some(alias.to_string())
        } else {
            None
        }
    })
}



fn determine_gre_interface(link_message: &LinkMessage) -> Result<NetlinkInterfaceKind, NetlinkConversionError> {
    fn extract_address(info_gre_tap: &[InfoGreTap], kind: u16) -> Option<Ipv4Addr> {
        info_gre_tap.iter().find_map(|info_gre_tap|{
            if let InfoGreTap::Other(nla) = info_gre_tap {
                if nla.kind() == kind {
                    let buffer: &mut [u8; 4] = &mut [0; 4];
                    nla.emit_value(buffer);
                    let octets: [u8; 4] = buffer[0..4].try_into().ok()?;
                    return Some(Ipv4Addr::from(octets))
                }
            }
            None
        })
    }
    
    struct GreAddresses {
        local: Ipv4Addr,
        remote: Ipv4Addr,
    }

    let gre_addresses = link_message.attributes.iter().find_map(|nla| {
        if let LinkAttribute::LinkInfo(link_info) = nla {
            link_info.iter().find_map(|link_info| {
                if let LinkInfo::Data(InfoData::GreTap(gretap)) = link_info {
                    let local_address = extract_address(gretap, 0x06);
                    let remote_address = extract_address(gretap, 0x07);
                    if let (Some(local), Some(remote)) = (local_address, remote_address) {
                        Some(GreAddresses { local, remote })
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
        } else {
            None
        }
    }).ok_or(NetlinkConversionError::NetworkInterfaceGre)?;

    Ok(NetlinkInterfaceKind::GreTap {
        local: gre_addresses.local,
        remote: gre_addresses.remote,
    })

}
