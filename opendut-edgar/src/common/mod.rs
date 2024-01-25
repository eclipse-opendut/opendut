use std::str::FromStr;

use opendut_types::util::net::NetworkInterfaceName;

pub mod carl;
pub mod settings;


pub fn default_bridge_name() -> NetworkInterfaceName {
    NetworkInterfaceName::from_str("br-opendut").unwrap()
}
