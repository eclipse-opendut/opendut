
fn main() -> Result<(), Box<dyn std::error::Error>> {
    std::env::set_var("PROTOC", protobuf_src::protoc());

    let protos = glob::glob("proto/**/*.proto")?.collect::<Result<Vec<_>, _>>()?;

    let includes = [
        "proto/",
        "../../opendut-types/proto/",
    ];

    tonic_build::configure()
        .extern_path(
            ".opendut.types",
            "opendut_types::proto"
        )
        .compile_protos(&protos, &includes)?;

    Ok(())
}
