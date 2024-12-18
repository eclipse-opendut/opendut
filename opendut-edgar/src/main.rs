#[tokio::main]
async fn main() -> anyhow::Result<()> {
    opendut_util::crypto::install_default_provider();

    opendut_edgar::cli().await
}
