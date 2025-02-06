fn main() {
    //collect build information for use at runtime
    shadow_rs::ShadowBuilder::builder()
        .build().unwrap();
}
