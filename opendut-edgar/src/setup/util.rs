use std::fs::File;
use std::io;
use std::io::Read;
use std::ops::Not;
use std::path::Path;
use std::process::{Command, Output};

use anyhow::{anyhow, bail, Context};
use sha2::{Digest, Sha256};

use crate::setup::User;

pub(crate) trait EvaluateRequiringSuccess {
    fn evaluate_requiring_success(&mut self) -> anyhow::Result<Output>;
}
impl EvaluateRequiringSuccess for Command {
    fn evaluate_requiring_success(&mut self) -> anyhow::Result<Output> {
        let output = self.output()?;

        if output.status.success().not() {
            let mut error = format!("Error while running `{self:?}`:\n");
            if let Some(status) = &output.status.code() {
                error += format!("  Status Code: {}\n", status).as_ref();
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

pub fn file_checksum(path: impl AsRef<Path>) -> Result<Vec<u8>, io::Error> {
    let file = File::open(path.as_ref())?;
    sha256_digest(file)
}

fn sha256_digest(mut reader: impl Read) -> Result<Vec<u8>, io::Error> {
    let mut hasher = Sha256::new();
    let _ = io::copy(&mut reader, &mut hasher)?;
    let hash = hasher.finalize();
    Ok(hash.to_vec())
}
