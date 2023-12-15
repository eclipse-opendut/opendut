use std::process::Command;

#[tracing::instrument(level = tracing::Level::TRACE)]
pub fn install_crate(name: &str) -> anyhow::Result<()> {
    Command::new("cargo")
        .arg("install")
        .arg(name)
        .run_requiring_success();
    Ok(())
}


pub(crate) trait RunRequiringSuccess {
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
