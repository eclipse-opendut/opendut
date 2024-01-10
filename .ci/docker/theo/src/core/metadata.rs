use std::path::PathBuf;

use cargo_toml::{Manifest, Value};

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

fn get_ci_package_version(manifest: &Manifest, package_name: &str) -> Option<String> {
    let metadata_table = manifest.workspace.clone().unwrap().metadata.unwrap();
    match metadata_table {
        Value::Table(table) => {
            let ci_table = table.get("ci").unwrap();
            match ci_table {
                Value::Table(ci_table) => {
                    let package_table = ci_table.get(package_name).unwrap();
                    match package_table {
                        Value::Table(package_table) => {
                            let package_version = package_table.get("version").unwrap();
                            match package_version {
                                Value::String(package_version) => {
                                    Some(package_version.to_string())
                                }
                                _ => { None }
                            }
                        }
                        _ => { None }
                    }
                }
                _ => { None }
            }
        }
        _ => { None }
    }
}

fn get_package_version(package_name: &str) -> String {
    let package_toml_path = PathBuf::project_path_buf().join(format!("{}/Cargo.toml", package_name));
    let package_manifest = Manifest::from_path(&package_toml_path).unwrap();
    let package_version = package_manifest.package.unwrap().version.unwrap();
    package_version
}

pub fn cargo_netbird_versions() -> Metadata {
    let workspace_cargo_toml_path = PathBuf::project_path_buf().join("Cargo.toml");
    let workspace_manifest = Manifest::from_path(&workspace_cargo_toml_path).unwrap();

    let versions = NetbirdMetadata {
        netbird_client_version: get_ci_package_version(&workspace_manifest, NetbirdApplicationNames::NetbirdClient.as_str()).unwrap(),
        netbird_signal_version: get_ci_package_version(&workspace_manifest, NetbirdApplicationNames::NetbirdSignal.as_str()).unwrap(),
        netbird_management_version: get_ci_package_version(&workspace_manifest, NetbirdApplicationNames::NetbirdManagement.as_str()).unwrap(),
        netbird_dashboard_version: get_ci_package_version(&workspace_manifest, NetbirdApplicationNames::NetbirdDashboard.as_str()).unwrap(),
    };
    let carl_version = get_package_version("opendut-carl");

    Metadata {
        netbird: versions,
        carl_version,
    }
}