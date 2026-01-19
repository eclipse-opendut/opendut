use std::env;
use std::ops::Not;
use std::path::PathBuf;

fn main() {
    if PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("shadow.rs").exists().not()
    || env::var_os("CARGO_SUPPRESS_SHADOW_REBUILD").is_none() {
        //collect build information for use at runtime
        shadow_rs::ShadowBuilder::builder()
            .build().unwrap();
    };
}
