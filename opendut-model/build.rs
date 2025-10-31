use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    unsafe {
        std::env::set_var("PROTOC", protobuf_src::protoc());
    }

    let protos = glob::glob("proto/**/*.proto")?.collect::<Result<Vec<_>, _>>()?;

    let includes = vec![
        PathBuf::from("proto/"),
    ];

    let mut builder = tonic_prost_build::Config::new();
    builder.type_attribute(".", "#[allow(clippy::all)]");


    // Only include protos for VIPER, when the feature flag i
    #[cfg(feature = "viper")]
    let includes = {
        let mut includes = includes;
        includes.push(
            repo_path::repo_path!("opendut-viper/viper-rt/proto/")
        );

        builder.extern_path(
            ".opendut.viper.rt",
            "::viper_rt::proto"
        );

        includes
    };
    #[cfg(not(feature = "viper"))]
    let protos = {
        let mut protos = protos;
        protos.retain(|proto_path|
            ! proto_path.starts_with("proto/opendut/model/viper/")
        );
        protos
    };


    builder
        .compile_protos(&protos, &includes)?;

    Ok(())
}
