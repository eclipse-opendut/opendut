use std::process::Command;

use tracing_subscriber::fmt::format::FmtSpan;
use crate::core::dependency::Crate;

use crate::core::types::Arch;

#[tracing::instrument(level = tracing::Level::TRACE)]
pub fn install_crate(install: Crate) -> anyhow::Result<()> {
    Command::new("cargo")
        .arg("install")
        .arg(install.ident())
        .run_requiring_success();
    Ok(())
}

#[tracing::instrument]
pub fn install_toolchain(arch: &Arch) -> anyhow::Result<()> {
    Command::new("rustup")
        .args(["target", "add", &arch.triple()])
        .run_requiring_success();
    Ok(())
}


pub trait RunRequiringSuccess {
    fn run_requiring_success(&mut self);
}
impl RunRequiringSuccess for Command {
    fn run_requiring_success(&mut self) {
        let status = self.status()
            .expect("Error while running command.");

        if !status.success() {
            let mut error = format!("Error while running command: {self:?}\n");
            if let Some(status) = &status.code() {
                error += format!("  Exited with status code {}.\n", status).as_ref();
            }
            panic!("{}", error)
        }
    }
}


pub fn init_tracing() -> anyhow::Result<()> {
    use tracing_subscriber::filter::{EnvFilter, LevelFilter};

    let tracing_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::DEBUG.into())
        .from_env()?
        .add_directive("opendut=trace".parse()?);

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .with_span_events(FmtSpan::ENTER | FmtSpan::CLOSE)
        .with_env_filter(tracing_filter)
        .compact()
        .init();
    Ok(())
}
