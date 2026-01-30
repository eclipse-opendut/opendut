use std::process::ExitCode;

#[tokio::main]
async fn main() -> anyhow::Result<ExitCode> {
    opendut_util::crypto::install_default_provider();

    opendut_edgar::cli().await
}
