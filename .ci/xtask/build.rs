fn main() -> shadow_rs::SdResult<()> {
    println!("cargo:rustc-env=RUST_BACKTRACE=1"); //enable backtraces by default

    shadow_rs::new() //collect build information for use at runtime
}
