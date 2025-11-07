use std::path::PathBuf;
use repo_path::repo_path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    unsafe {
        std::env::set_var("PROTOC", protobuf_src::protoc());
    }

    let protos = glob::glob("proto/**/*.proto")?.collect::<Result<Vec<_>, _>>()?;

    let includes = [
        PathBuf::from("proto/"),
        repo_path!("opendut-model/proto/"),
        repo_path!("opendut-viper/viper-rt/proto/"),
    ];

    tonic_prost_build::configure()
        .extern_path(
            ".opendut.model",
            "::opendut_model::proto"
        )
        .extern_path(
            ".opendut.viper.rt",
            "::opendut_viper_rt::proto"
        )
        .type_attribute(".", "#[allow(clippy::all)]")
        .compile_protos(&protos, &includes)?;

    Ok(())
}
