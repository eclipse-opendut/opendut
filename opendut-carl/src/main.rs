#[tokio::main]
async fn main() -> anyhow::Result<()> {
    opendut_carl::cli().await
}
