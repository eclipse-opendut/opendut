use std::str::FromStr;

use opendut_model::util::net::NetworkInterfaceName;

pub mod carl;
pub mod settings;
pub mod task;

pub fn default_bridge_name() -> NetworkInterfaceName {
    NetworkInterfaceName::from_str("br-opendut").unwrap()
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
