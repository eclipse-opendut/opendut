use std::path::PathBuf;

use anyhow::Context;
use opendut_util::project;

use crate::common::constants::edgar_install_directory;


pub fn executable_install_path() -> anyhow::Result<PathBuf> {
    let executable_path = std::env::current_exe()?;
    let executable_name = executable_path.file_name()
        .context("Failed to retrieve file name of executable.")?;

    Ok(edgar_install_directory().join(executable_name))
}

/// Directory where we will link our executable to make it available as
/// a command without specifying a path, so that it can be run like this:
/// $ opendut-edgar --help
#[allow(non_snake_case)]
pub fn default_PATH_dir() -> PathBuf { PathBuf::from("/usr/bin/") }


pub fn passwd_file() -> PathBuf { PathBuf::from("/etc/passwd") }


pub fn systemd_service_dir() -> PathBuf { PathBuf::from("/etc/systemd/system/") }
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
    use super::*;
    use tokio::process::Command;

    pub fn path_in_edgar_distribution() -> anyhow::Result<PathBuf> {
        let path = PathBuf::from("install/netbird.tar.gz");
        project::make_path_absolute(&path)
            .context(format!("Failed to determine absolute path of NetBird in the unpacked EDGAR distribution, which is supposed to be at '{path:?}'"))
    }

    pub fn unpack_dir() -> anyhow::Result<PathBuf> {
        let path = edgar_install_directory().join("netbird");
        project::make_path_absolute(&path)
            .context(format!("Failed to determine absolute path where NetBird should be unpacked to, which is supposed to be at {path:?}"))
    }

    pub fn netbird_binary_file() -> PathBuf {
        edgar_install_directory().join("netbird").join("netbird")
    }

    pub fn default_checksum_unpack_file() -> PathBuf {
        edgar_install_directory().join("netbird.tar.gz.checksum")
    }

    // The directory where the accompanying installation files of the distribution are copied for comparison.
    pub fn default_installation_companion_directory() -> PathBuf {
        edgar_install_directory().join("install")
    }

    pub fn command() -> anyhow::Result<Command> {
        let executable = unpack_dir()?.join("netbird");
        let mut command = Command::new(executable);
        command.env("SSL_CERT_FILE", default_os_cert_store_ca_certificate_path());
        Ok(command)
    }
}

pub mod rperf {
    use super::*;

    pub fn path_in_edgar_distribution() -> anyhow::Result<PathBuf> {
        let path = PathBuf::from("install/rperf");
        project::make_path_absolute(&path)
            .context(format!("Failed to determine absolute path of rperf in the unpacked EDGAR distribution, which is supposed to be at '{path:?}'"))
    }
}

pub const REQUIRED_COMMAND_LINE_PROGRAMS_SERVICE: [(&str, &str); 1] = [("systemctl", "--version")];
pub const REQUIRED_COMMAND_LINE_PROGRAMS_CAN: [(&str, &str); 2] = [("cannelloni", "-h"), ("cangw", "-s")];
