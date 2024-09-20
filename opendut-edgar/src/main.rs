#[tokio::main]
async fn main() -> anyhow::Result<()> {
    opendut_edgar::cli().await
}
