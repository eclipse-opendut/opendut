use std::path::PathBuf;

use anyhow::Context;

pub const USER_NAME: &str = "opendut-network";

pub fn executable_install_path() -> anyhow::Result<PathBuf> {
    let executable_path = std::env::current_exe()?;
    let executable_name = executable_path.file_name()
        .context("Failed to retrieve file name of executable.")?;

    let install_dir = PathBuf::from("/opt/opendut-network/edgar/");

    Ok(install_dir.join(executable_name))
}

#[allow(non_snake_case)]
pub fn PATH_dir() -> PathBuf { PathBuf::from("/usr/bin/") }

pub const SYSTEMD_SERVICE_FILE_NAME: &str = "opendut-edgar.service";

pub fn default_config_merge_suggestion_file_path() -> PathBuf {
    PathBuf::from("/etc/opendut-network/edgar-merge-suggestion.toml")
}

pub fn required_kernel_modules() -> Vec<String> {
    vec![
        "can-gw".to_string(), 
        "vcan".to_string()
        ]
}

pub mod netbird {
    use std::path::PathBuf;

    use anyhow::anyhow;

    use opendut_util::project;

    pub fn path_in_edgar_distribution() -> anyhow::Result<PathBuf> {
        let path = "install/netbird.tar.gz";
        project::make_path_absolute(path)
            .map_err(|cause| anyhow!("Failed to determine absolute path of NetBird in the unpacked EDGAR distribution, which is supposed to be at '{path}': {cause}"))
    }

    pub fn unpack_dir() -> anyhow::Result<PathBuf> {
        let path = "/opt/opendut-network/edgar/netbird/";
        project::make_path_absolute(path)
            .map_err(|cause| anyhow!("Failed to determine absolute path where NetBird should be unpacked to, which is supposed to be at '{path}': {cause}"))
    }

    pub fn checksum_unpack_file() -> PathBuf {
        PathBuf::from("/opt/opendut-network/edgar/netbird.tar.gz.checksum")
    }

    pub fn unpacked_executable() -> anyhow::Result<PathBuf> {
        unpack_dir().map(|dir| dir.join("netbird"))
    }
}
