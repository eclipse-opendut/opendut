use crate::fs;
use std::os::unix;
use std::process::Command;

use anyhow::{Context, Result};

use crate::setup::constants::{executable_install_path, PATH_dir, SYSTEMD_SERVICE_FILE_NAME};
use crate::setup::task::{Success, Task, TaskFulfilled};
use crate::setup::util;

pub struct CopyExecutable;
impl Task for CopyExecutable {
    fn description(&self) -> String {
        format!("Copy executable to \"{}\"", executable_install_path().unwrap().display())
    }
    fn check_fulfilled(&self) -> Result<TaskFulfilled> {
        let installed_path = executable_install_path()?;
        if installed_path.exists() {
            let installed_digest = util::checksum::file(installed_path)?;
            let unpacked_path = std::env::current_exe()?;
            let unpacked_digest = util::checksum::file(unpacked_path)?;

            if installed_digest == unpacked_digest {
                return Ok(TaskFulfilled::Yes);
            }
        }
        Ok(TaskFulfilled::No)
    }
    fn execute(&self) -> Result<Success> {
        let target_path = executable_install_path()?;
        fs::create_dir_all(target_path.parent().unwrap())?;

        let executable_path = std::env::current_exe()?;
        let executable_name = executable_path.file_name()
            .context("Failed to retrieve file name of executable.")?;

        // Stop service to allow replacing executable. This also ensures the new executable is used in the service immediately.
        let mut command = Command::new("systemctl");
        let command = command.arg("stop").arg(SYSTEMD_SERVICE_FILE_NAME);
        let _ = command.output(); //ignore errors, e.g. when service is not running.

        fs::copy(&executable_path, &target_path)
            .context(format!("Error while copying file from '{}' to '{}'", executable_path.display(), target_path.display()))?;

        let link_path = PATH_dir().join(executable_name);
        if link_path.exists() {
            fs::remove_file(&link_path)?;
        }
        unix::fs::symlink(&target_path, &link_path)
            .context(format!("Error while linking executable from '{}' to PATH directory '{}'", target_path.display(), link_path.display()))?;

        Ok(Success::default())
    }
}
