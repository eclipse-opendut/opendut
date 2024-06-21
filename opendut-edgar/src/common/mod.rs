use std::str::FromStr;

use opendut_types::util::net::NetworkInterfaceName;

pub mod carl;
pub mod settings;


pub fn default_bridge_name() -> NetworkInterfaceName {
    NetworkInterfaceName::from_str("br-opendut").unwrap()
}

pub fn default_can_bridge_name() -> NetworkInterfaceName {
    NetworkInterfaceName::from_str("br-vcan-opendut").unwrap()
}

pub mod constants {
    use std::path::PathBuf;

    pub fn edgar_install_directory() -> PathBuf {
        PathBuf::from("/opt/opendut/edgar/")
    }

    pub mod rperf {
        use std::path::PathBuf;

        pub fn executable_install_file() -> PathBuf {
            let install_dir = crate::common::constants::edgar_install_directory();
            install_dir.join("rperf")
        }
    }
}