use std::{fs, io};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::process::{Command, Output};

use anyhow::{anyhow, bail, Context};
use nix::unistd::{Group, User};
use sha2::{Digest, Sha256};

use crate::setup::constants;

pub fn evaluate_requiring_success(command: &mut Command) -> anyhow::Result<Output> {
    let output = command.output()?;

    if !output.status.success() {
        let mut error = format!("Error while running `{command:?}`:\n");
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

pub fn chown(path: impl AsRef<Path>) -> anyhow::Result<()> {
    let path = path.as_ref();
    let name = constants::USER_NAME;

    let user = User::from_name(name)?.ok_or(anyhow!("No user '{}' found.", name))?;
    let group = Group::from_name(name)?.ok_or(anyhow!("No group '{}' found.", name))?;

    nix::unistd::chown(path, Some(user.uid), Some(group.gid))
        .context(format!("Failed to set owner of '{}' to user '{}'.", path.display(), name))?;
    Ok(())
}

pub fn prepend_line(line: impl AsRef<str>, file: impl AsRef<Path>) -> anyhow::Result<()> {
    let line = line.as_ref();
    let file = file.as_ref();

    let file_content = fs::read_to_string(&file)
        .context(format!("Failed to read content of file '{}' while prepending line '{line}'.", file.display()))?;

    let new_file_content = format!("{line}\n{file_content}");

    fs::write(&file, new_file_content)
        .context(format!("Failed to write content of file '{}' while prepending line '{line}'.", file.display()))?;
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
