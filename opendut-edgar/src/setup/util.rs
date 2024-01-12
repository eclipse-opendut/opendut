use std::{fs, io};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::process::{Command, Output};

use anyhow::{anyhow, bail, Context};
use nix::unistd::{Group, User};
use sha2::{Digest, Sha256};

use crate::setup::constants;

pub(crate) trait EvaluateRequiringSuccess {
    fn evaluate_requiring_success(&mut self) -> anyhow::Result<Output>;
}
impl EvaluateRequiringSuccess for Command {
    fn evaluate_requiring_success(&mut self) -> anyhow::Result<Output> {
        let output = self.output()?;

        if !output.status.success() {
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

    let file_content = if file.exists() {
        fs::read_to_string(file)
            .context(format!("Failed to read content of file '{}' while prepending line '{line}'.", file.display()))?
    } else {
        fs::create_dir_all(file.parent().unwrap())?;
        String::new()
    };

    let new_file_content = format!("{line}\n{file_content}");

    fs::write(file, new_file_content)
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


#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::prelude::*;
    use assert_fs::TempDir;

    #[test]
    fn should_prepend_line() -> anyhow::Result<()> {
        let temp = TempDir::new()?;

        let test_file = temp.child("test_file");
        let previous_content = "bbb";
        test_file.write_str(previous_content)?;

        let line_to_prepend = "aaa";

        prepend_line(line_to_prepend, &test_file)?;

        let result = fs::read_to_string(test_file)?;
        assert_eq!(result, format!("{line_to_prepend}\n{previous_content}"));

        Ok(())
    }

    #[test]
    fn should_prepend_line_when_file_not_exists() -> anyhow::Result<()> {
        let temp = TempDir::new()?;

        let test_file = temp.child("test_file");

        let line_to_prepend = "aaa";

        prepend_line(line_to_prepend, &test_file)?;

        let result = fs::read_to_string(test_file)?;
        assert_eq!(result, format!("{line_to_prepend}\n"));

        Ok(())
    }
}
