fn main() {
    println!("cargo:rustc-env=RUST_BACKTRACE=1"); //enable backtraces by default

    //collect build information for use at runtime
    shadow_rs::ShadowBuilder::builder()
        .build().unwrap();
}
