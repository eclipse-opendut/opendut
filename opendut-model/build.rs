
fn main() -> Result<(), Box<dyn std::error::Error>> {
    unsafe {
        std::env::set_var("PROTOC", protobuf_src::protoc());
    }

    let protos = glob::glob("proto/**/*.proto")?.collect::<Result<Vec<_>, _>>()?;

    let includes = [
        "proto/"
    ];

    tonic_prost_build::Config::new()
        .type_attribute(".", "#[allow(clippy::all)]")
        .compile_protos(&protos, &includes)?;

    Ok(())
}
