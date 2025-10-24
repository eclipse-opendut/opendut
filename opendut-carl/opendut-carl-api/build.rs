use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    unsafe {
        std::env::set_var("PROTOC", protobuf_src::protoc());
    }

    let protos = glob::glob("proto/**/*.proto")?.collect::<Result<Vec<_>, _>>()?;

    let includes = [
        PathBuf::from("proto/"),
        PathBuf::from("../../opendut-model/proto/"),
    ];

    tonic_prost_build::configure()
        .extern_path(
            ".opendut.types",
            "opendut_model::proto"
        )
        .type_attribute(".", "#[allow(clippy::all)]")
        .compile_protos(&protos, &includes)?;

    Ok(())
}
