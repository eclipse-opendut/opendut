use std::fs;
use std::ops::Not;
use std::path::PathBuf;

use cargo_metadata::MetadataCommand;

use opendut_util::project;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    unsafe {
        std::env::set_var("PROTOC", protobuf_src::protoc());
    }

    let workspace_dir = project::workspace_dir(env!("CARGO"))?;

    let metadata = MetadataCommand::new()
        .manifest_path(workspace_dir.join("Cargo.toml"))
        .exec()?;
    let version = metadata.workspace_metadata["ci"]["netbird"]["version"].as_str()
        .ok_or("NetBird version not defined.")?;
    let netbird_proto_url = metadata.workspace_metadata["ci"]["netbird"]["protobuf"].as_str()
        .ok_or("NetBird protobuf URL not defined.")?;

    let proto_dir = PathBuf::from("proto/").join(format!("netbird-v{version}"));
    fs::create_dir_all(&proto_dir)?;

    let proto_file = proto_dir.join("daemon.proto");

    if proto_file.exists().not() {
        println!("Downloading NetBird proto file...");

        let bytes = reqwest::blocking::get(netbird_proto_url)?
            .error_for_status()?
            .bytes()?;

        fs::write(&proto_file, bytes)
            .map_err(|cause| format!("Error while writing to {proto_file:?}: {cause}"))?;
    }

    let protos = [proto_file];
    let includes = [proto_dir];

    tonic_prost_build::configure()
        .build_server(false)
        .type_attribute(".", "#[allow(clippy::all)]")
        .compile_protos(&protos, &includes)?;

    Ok(())
}
