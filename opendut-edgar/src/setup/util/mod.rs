pub mod checksum;
mod dry_run;
pub use dry_run::DryRun;

use std::fs::File;
use std::io;
use std::io::Write;
use std::ops::Not;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::{Command, Output};

use anyhow::{anyhow, bail, Context};
use cfg_if::cfg_if;
use crate::fs;
use crate::setup::User;

pub trait CommandRunner: Send + Sync {
    fn run(&self, command: &mut Command) -> anyhow::Result<Output>;
}
pub struct DefaultCommandRunner;
impl CommandRunner for DefaultCommandRunner {
    fn run(&self, command: &mut Command) -> anyhow::Result<Output> {
        command.evaluate_requiring_success()
    }
}
cfg_if! {
     if #[cfg(test)] {
        pub struct NoopCommandRunner;
        impl CommandRunner for NoopCommandRunner {
            fn run(&self, _command: &mut Command) -> anyhow::Result<Output> {
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

pub fn create_file_and_ensure_it_can_only_be_read_or_modified_by_owner(target: &Path) -> anyhow::Result<()> {
    if target.exists().not() {
        File::create(target)?;
    }
    let read_only_permissions = std::fs::Permissions::from_mode(0o600);
    fs::set_permissions(target, read_only_permissions)?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_restricted_file() {
        // Arrange: assumes to be run in unix
        // 100 means it's a regular file
        // 600 means the owner may read and write that file and nobody else
        let expected_permissions = std::fs::Permissions::from_mode(0o100600 );
        let temp_dir = tempfile::TempDir::new().expect("failed to create temp dir");
        let path = temp_dir.path().join("edgar.toml");
        assert!(path.exists().not());

        // Act: create file
        create_file_and_ensure_it_can_only_be_read_or_modified_by_owner(&path).unwrap();

        // Assert:
        let file = File::open(path).unwrap();
        let permissions = file.metadata().unwrap().permissions();
        assert_eq!(permissions, expected_permissions);
    }
}
