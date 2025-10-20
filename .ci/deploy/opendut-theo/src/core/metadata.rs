use std::path::PathBuf;

use cargo_toml::Manifest;

use crate::core::project::ProjectRootDir;


pub(crate) fn get_package_version(package_name: &str) -> String {
    let package_toml_path = PathBuf::project_path_buf().join(format!("{package_name}/Cargo.toml"));
    let package_manifest = Manifest::from_path(package_toml_path).unwrap();
    package_manifest.package.unwrap().version.unwrap()
}
