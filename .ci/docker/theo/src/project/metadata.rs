use std::path::Path;
use crate::project::project_root_dir;

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
pub struct NetbirdMetadata {
    pub netbird_client_version: String,
    pub netbird_signal_version: String,
    pub netbird_management_version: String,
    pub netbird_dashboard_version: String,
}

pub fn cargo_netbird_versions() -> NetbirdMetadata {
    let cargo_toml_path = Path::new(&project_root_dir()).join("Cargo.toml");
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
    versions
}