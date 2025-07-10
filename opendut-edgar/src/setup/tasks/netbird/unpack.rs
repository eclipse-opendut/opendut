use crate::fs;
use crate::fs::File;
use std::path::{PathBuf};

use anyhow::{Context, Result};
use async_trait::async_trait;
use flate2::read::GzDecoder;

use crate::setup::{constants, util};
use crate::common::task::{Success, Task, TaskStateFulfilled};

pub struct Unpack {
    netbird_archive: PathBuf,
    netbird_binary: PathBuf,
    to_dir: PathBuf,
    checksum_unpack_file: PathBuf,
    installation_companion_directory: PathBuf,
}

impl Unpack {
    fn unpack_netbird_archive(&self) -> Result<()> {
        let archive = File::open(&self.netbird_archive)
            .context(format!("Failed to open NetBird distribution file at '{}'.", self.netbird_archive.display()))?;
        let archive = GzDecoder::new(archive);
        let mut archive = tar::Archive::new(archive);
        archive.set_preserve_permissions(true);

        fs::create_dir_all(&self.to_dir)?;
        archive.unpack(&self.to_dir)?;
        
        Ok(())
    }

    fn copy_netbird_checksum_file(&self) -> Result<()> {
        let checksum = util::checksum::file(&self.netbird_archive)?;
        let checksum_unpack_file = &self.checksum_unpack_file;
        fs::create_dir_all(checksum_unpack_file.parent().unwrap())?;
        fs::write(checksum_unpack_file, checksum)
            .context(format!("Writing checksum for netbird.tar.gz to '{}'.", checksum_unpack_file.display()))?;

        Ok(())
    }

    fn copy_netbird_tarball_to_installation_companion_directory(&self) -> Result<()> {
        fs::create_dir_all(&self.installation_companion_directory)?;
        let target = self.installation_companion_directory.join("netbird.tar.gz");
        fs::copy(&self.netbird_archive, &target)
            .context(format!("Failed to copy netbird tarball to installation companion directory at {}.", target.display()))?;
        
        Ok(())
    }
}

#[async_trait]
impl Task for Unpack {
    fn description(&self) -> String {
        String::from("NetBird - Unpack")
    }
    async fn check_present(&self) -> Result<TaskStateFulfilled> {

        let unpacked_checksum_file = &self.checksum_unpack_file;
        if !self.netbird_archive.exists() && self.netbird_binary.exists() {
            // allow running setup even if the netbird archive is not present, but the binary is already unpacked.
            return Ok(TaskStateFulfilled::Unchecked);
        } 

        if unpacked_checksum_file.exists() {
            let installed_digest = fs::read(unpacked_checksum_file)?;
            let distribution_digest = util::checksum::file(&self.netbird_archive)?;

            if installed_digest == distribution_digest {
                return Ok(TaskStateFulfilled::Yes);
            }
        }
        Ok(TaskStateFulfilled::No)
    }
    async fn make_present(&self) -> Result<Success> {
        if !self.netbird_archive.exists() && self.netbird_binary.exists() {
            Ok(Success::message("NetBird archive not found, but binary already unpacked. Skipping unpacking."))
        } else {
            self.unpack_netbird_archive()?;
            self.copy_netbird_checksum_file()?;
            self.copy_netbird_tarball_to_installation_companion_directory()?;
            Ok(Success::default())
        }
    }
}
impl Default for Unpack {
    fn default() -> Self {
        Self {
            netbird_archive: constants::netbird::path_in_edgar_distribution().unwrap(),
            netbird_binary: constants::netbird::netbird_binary_file(),
            to_dir: constants::netbird::unpack_dir().unwrap(),
            checksum_unpack_file: constants::netbird::default_checksum_unpack_file(),
            installation_companion_directory: constants::netbird::default_installation_companion_directory(),
        }
    }
}

#[cfg(test)]
mod tests {
    use assert_fs::prelude::*;
    use assert_fs::TempDir;
    
    use crate::common::task::{Task, TaskStateFulfilled};
    use crate::setup::tasks::netbird::Unpack;
    use crate::setup::util;

    #[tokio::test]
    async fn should_check_task_is_fulfilled() -> anyhow::Result<()> {
        let temp = TempDir::new()?;

        let from = temp.child("netbird.tar.gz");
        from.write_binary(b"hello")?;

        let to_dir = temp.child("netbird");

        let checksum_file_path = temp.child("netbird.tar.gz.checksum");

        let task = Unpack {
            netbird_archive: from.to_path_buf(),
            netbird_binary: Default::default(),
            to_dir: to_dir.to_path_buf(),
            checksum_unpack_file: checksum_file_path.to_path_buf(),
            installation_companion_directory: Default::default(),
        };

        assert_eq!(task.check_present().await?, TaskStateFulfilled::No);
        
        checksum_file_path.write_binary(&util::checksum::file(from.path())?)?;

        let task = Unpack {
            netbird_archive: from.to_path_buf(),
            netbird_binary: Default::default(),
            to_dir: to_dir.to_path_buf(),
            checksum_unpack_file: checksum_file_path.to_path_buf(),
            installation_companion_directory: Default::default(),
        };

        assert_eq!(task.check_present().await?, TaskStateFulfilled::Yes);

        Ok(())
    }
}
