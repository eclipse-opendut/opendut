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
