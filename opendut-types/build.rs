use std::io::Result;

fn main() -> Result<()> {
    std::env::set_var("PROTOC", protobuf_src::protoc());

    let protos = [
        "proto/opendut/types/cluster/cluster.proto",
        "proto/opendut/types/peer/peer.proto",
        "proto/opendut/types/peer/configuration.proto",
        "proto/opendut/types/peer/executor.proto",
        "proto/opendut/types/topology/device.proto",
        "proto/opendut/types/topology/topology.proto",
        "proto/opendut/types/util/metadata.proto",
        "proto/opendut/types/util/net.proto",
        "proto/opendut/types/util/uuid.proto",
        "proto/opendut/types/vpn/vpn.proto",
    ];

    let includes = [
        "proto/"
    ];

    prost_build::compile_protos(&protos, &includes)
}
