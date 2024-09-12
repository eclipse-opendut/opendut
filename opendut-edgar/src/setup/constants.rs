use std::path::PathBuf;

use anyhow::Context;
use crate::common::constants::edgar_install_directory;


pub fn executable_install_path() -> anyhow::Result<PathBuf> {
    let executable_path = std::env::current_exe()?;
    let executable_name = executable_path.file_name()
        .context("Failed to retrieve file name of executable.")?;

    Ok(edgar_install_directory().join(executable_name))
}

#[allow(non_snake_case)]
pub fn PATH_dir() -> PathBuf { PathBuf::from("/usr/bin/") }

pub const SYSTEMD_SERVICE_FILE_NAME: &str = "opendut-edgar.service";

pub const KERNEL_MODULE_LOAD_RULE_PREFIX: &str = "opendut-edgar";

pub fn default_carl_ca_certificate_path() -> PathBuf {
    PathBuf::from("/etc/opendut/tls/ca.pem")
}
pub fn default_checksum_carl_ca_certificate_file() -> PathBuf {
    PathBuf::from("/etc/opendut/tls/.ca.pem.checksum")
}
pub fn default_os_cert_store_ca_certificate_path() -> PathBuf {
    PathBuf::from("/usr/local/share/ca-certificates/opendut-ca.crt")
}
pub fn default_checksum_os_cert_store_ca_certificate_file() -> PathBuf {
    PathBuf::from("/usr/local/share/ca-certificates/.opendut-ca.crt.checksum")
}

pub fn default_config_merge_suggestion_file_path() -> PathBuf {
    PathBuf::from("/etc/opendut/edgar-merge-suggestion.toml")
}


pub mod netbird {
    use std::path::PathBuf;

    use anyhow::anyhow;

    use opendut_util::project;

    use crate::common::constants::edgar_install_directory;

    pub fn path_in_edgar_distribution() -> anyhow::Result<PathBuf> {
        let path = PathBuf::from("install/netbird.tar.gz");
        project::make_path_absolute(&path)
            .map_err(|cause| anyhow!("Failed to determine absolute path of NetBird in the unpacked EDGAR distribution, which is supposed to be at '{path:?}': {cause}"))
    }

    pub fn unpack_dir() -> anyhow::Result<PathBuf> {
        let path = edgar_install_directory().join("netbird");
        project::make_path_absolute(&path)
            .map_err(|cause| anyhow!("Failed to determine absolute path where NetBird should be unpacked to, which is supposed to be at {path:?}: {cause}"))
    }

    pub fn default_checksum_unpack_file() -> PathBuf {
        PathBuf::from("/opt/opendut/edgar/netbird.tar.gz.checksum")
    }

    pub fn unpacked_executable() -> anyhow::Result<PathBuf> {
        unpack_dir().map(|dir| dir.join("netbird"))
    }
}

pub mod rperf {
    use std::path::PathBuf;
    use anyhow::anyhow;
    use opendut_util::project;

    pub fn path_in_edgar_distribution() -> anyhow::Result<PathBuf> {
        let path = PathBuf::from("install/rperf");
        project::make_path_absolute(&path)
            .map_err(|cause| anyhow!("Failed to determine absolute path of rperf in the unpacked EDGAR distribution, which is supposed to be at '{path:?}': {cause}"))
    }
}

pub const REQUIRED_COMMAND_LINE_PROGRAMS: [(&str, &str); 3] = [("systemctl", "--version"), ("cannelloni", "-h"), ("cangw", "-s")];
