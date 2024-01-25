

fn main() -> Result<(), Box<dyn std::error::Error>> {
    std::env::set_var("PROTOC", protobuf_src::protoc());

    let protos = [
        "proto/opendut/carl/services/cluster-manager.proto",
        "proto/opendut/carl/services/metadata-provider.proto",
        "proto/opendut/carl/services/peer-manager.proto",
        "proto/opendut/carl/services/peer-messaging-broker.proto",
        "proto/opendut/carl/services/resources-manager.proto",
    ];

    let includes = [
        "proto/",
        "../../opendut-types/proto/",
    ];

    tonic_build::configure()
        .extern_path(
            ".opendut.types",
            "opendut_types::proto"
        )
        .compile(&protos, &includes)?;

    Ok(())
}
