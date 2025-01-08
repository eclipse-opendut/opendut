use std::sync::LazyLock;
use cargo_metadata::Package;

static CARGO_METADATA: LazyLock<cargo_metadata::Metadata> = LazyLock::new(||
    cargo_metadata::MetadataCommand::new()
        .manifest_path(crate::constants::workspace_dir().join("Cargo.toml"))
        .exec()
        .expect("Failed to gather Cargo metadata.")
);

pub fn cargo() -> cargo_metadata::Metadata {
    CARGO_METADATA.to_owned()
}

pub fn repository_url() -> String {
    let carl_package: Package = cargo().workspace_packages().into_iter()
        .find(|&package| package.name == "opendut-carl")
        .expect("Could not extract repository url for package opendut-carl from opendut-carl/Cargo.toml.").to_owned();
    carl_package.repository.unwrap()
}
