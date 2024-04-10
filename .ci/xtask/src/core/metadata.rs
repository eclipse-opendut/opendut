use cargo_metadata::Package;
use lazy_static::lazy_static;

lazy_static! {
    static ref CARGO_METADATA: cargo_metadata::Metadata = {
        cargo_metadata::MetadataCommand::new()
            .manifest_path(crate::constants::workspace_dir().join("Cargo.toml"))
            .exec()
            .expect("Failed to gather Cargo metadata.")
    };
}

pub fn cargo() -> cargo_metadata::Metadata {
    CARGO_METADATA.to_owned()
}

pub fn repository_url() -> String {
    let carl_package: Package = cargo().workspace_packages().into_iter()
        .find(|&package| package.name == "opendut-carl").cloned()
        .expect("Could not extract repository url for package opendut-carl from opendut-carl/Cargo.toml.").to_owned();
    carl_package.repository.unwrap()
}
