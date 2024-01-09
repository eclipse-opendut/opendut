use std::path::PathBuf;

use crate::core::project::ProjectRootDir;

pub enum NetbirdApplicationNames {
    NetbirdClient,
    NetbirdManagement,
    NetbirdSignal,
    NetbirdDashboard,
}

impl NetbirdApplicationNames {
    fn as_str(&self) -> &'static str {
        match self {
            NetbirdApplicationNames::NetbirdClient => "netbird",
            NetbirdApplicationNames::NetbirdManagement => "netbird-management",
            NetbirdApplicationNames::NetbirdSignal => "netbird-signal",
            NetbirdApplicationNames::NetbirdDashboard => "netbird-dashboard",
        }
    }

}

impl std::fmt::Display for NetbirdApplicationNames {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug)]
pub struct Metadata {
    pub netbird: NetbirdMetadata,
    pub carl_version: String,
}


#[derive(Debug)]
pub struct NetbirdMetadata {
    pub netbird_client_version: String,
    pub netbird_signal_version: String,
    pub netbird_management_version: String,
    pub netbird_dashboard_version: String,
}

pub fn cargo_netbird_versions() -> Metadata {
    let cargo_toml_path = PathBuf::project_path_buf().join("Cargo.toml");
    let metadata = cargo_metadata::MetadataCommand::new().manifest_path(cargo_toml_path).exec().expect("Failed to gather Cargo metadata.");

    fn get_version(metadata: &cargo_metadata::Metadata, package_name: &str) -> String {
        metadata.workspace_metadata["ci"][package_name]["version"].as_str()
            .unwrap_or_else(|| panic!("No version information for dependency '{}' in root Cargo.toml.", package_name)).into()
    }

    let versions = NetbirdMetadata {
        netbird_client_version: get_version(&metadata, NetbirdApplicationNames::NetbirdClient.as_str()),
        netbird_signal_version: get_version(&metadata, NetbirdApplicationNames::NetbirdSignal.as_str()),
        netbird_management_version: get_version(&metadata, NetbirdApplicationNames::NetbirdManagement.as_str()),
        netbird_dashboard_version: get_version(&metadata, NetbirdApplicationNames::NetbirdDashboard.as_str()),
    };
    let carl_version = metadata.packages.iter().find(|package| package.name == "opendut-carl").unwrap().version.to_string();

    Metadata {
        netbird: versions,
        carl_version,
    }
}