pub mod checksum;
mod dry_run;
pub use dry_run::DryRun;

use std::io;
use std::io::Write;
use std::ops::Not;
use std::path::Path;
use std::process::{Command, Output};

use anyhow::{anyhow, bail, Context};
use crate::setup::User;

#[derive(Clone, Copy)]
pub enum CommandRunner {
    Default,
    #[cfg(test)]
    Noop,
}
impl CommandRunner {
    pub fn run(&self, command: &mut Command) -> anyhow::Result<Output> {
        match self {
            CommandRunner::Default => {
                command.evaluate_requiring_success()
            }
            #[cfg(test)]
            CommandRunner::Noop => {
                //do nothing
                Ok(Output {
                    status: std::process::ExitStatus::default(),
                    stdout: b"dummy".to_vec(),
                    stderr: b"dummy".to_vec(),
                })
            }
        }
    }
}

pub(crate) trait EvaluateRequiringSuccess {
    fn evaluate_requiring_success(&mut self) -> anyhow::Result<Output>;
}
impl EvaluateRequiringSuccess for Command {
    fn evaluate_requiring_success(&mut self) -> anyhow::Result<Output> {
        let output = self.output()?;

        if output.status.success().not() {
            let mut error = format!("Error while running `{self:?}`:\n");
            if let Some(status) = &output.status.code() {
                error += format!("  Status Code: {status}\n").as_ref();
            }
            if !output.stdout.is_empty() {
                error += format!("  Stdout: {}\n", String::from_utf8(output.stdout.clone())?).as_str();
            }
            if !output.stderr.is_empty() {
                error += format!("  Stderr: {}\n", String::from_utf8(output.stderr.clone())?).as_str();
            }
            bail!(error)
        }
        Ok(output)
    }
}

pub fn chown(user: &User, path: impl AsRef<Path>) -> anyhow::Result<()> {
    let path = path.as_ref();
    let name = &user.name;

    let user = nix::unistd::User::from_name(name)?
        .ok_or(anyhow!("No user '{}' found.", name))?;
    let group = nix::unistd::Group::from_name(name)?
        .ok_or(anyhow!("No group '{}' found.", name))?;

    nix::unistd::chown(path, Some(user.uid), Some(group.gid))
        .context(format!("Failed to set owner of '{}' to user '{}'.", path.display(), name))?;
    Ok(())
}



pub fn running_in_docker() -> bool {
    Path::new("/.dockerenv").exists()
}


pub fn user_confirmation_prompt(question: &str) -> anyhow::Result<bool> {
    print!("{question} [Y/n] ");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    match input.trim().to_lowercase().as_ref() {
        "" | "y" | "yes" => Ok(true),
        _ => Ok(false),
    }
}
