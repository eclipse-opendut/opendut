use std::fs;
use std::path::PathBuf;

use cargo_metadata::MetadataCommand;

use opendut_util::project;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    std::env::set_var("PROTOC", protobuf_src::protoc());

    let workspace_dir = project::workspace_dir(env!("CARGO"))?;

    let metadata = MetadataCommand::new()
        .manifest_path(workspace_dir.join("Cargo.toml"))
        .exec()?;
    let version = metadata.workspace_metadata["ci"]["netbird"]["version"].as_str()
        .ok_or("NetBird version not defined.")?;
    let netbird_proto_url = metadata.workspace_metadata["ci"]["netbird"]["protobuf"].as_str()
        .ok_or("NetBird protobuf url not defined.")?;

    let proto_dir = PathBuf::from("proto/").join(format!("netbird-v{version}"));
    let file_names = ["daemon.proto"];

    if !proto_dir.exists() {
        println!("Downloading proto files...");
        fs::create_dir_all(&proto_dir)?;

        for file_name in file_names {
            let bytes = reqwest::blocking::get(netbird_proto_url)?
                .error_for_status()?
                .bytes()?;

            let target_file = proto_dir.join(file_name);

            fs::write(&target_file, bytes)
                .map_err(|cause| format!("Error while writing to '{}': {cause}", target_file.display()))?;
        }
    }

    let protos = file_names.map(|file_name| proto_dir.join(file_name));

    let includes = [proto_dir];

    tonic_build::configure()
        .build_server(false)
        .compile_protos(&protos, &includes)?;

    Ok(())
}
