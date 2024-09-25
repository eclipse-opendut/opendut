use crate::fs;
use crate::fs::File;
use std::path::{PathBuf};

use anyhow::{Context, Result};
use async_trait::async_trait;
use flate2::read::GzDecoder;

use crate::setup::{constants, util};
use crate::common::task::{Success, Task, TaskFulfilled};

pub struct Unpack {
    from: PathBuf,
    to_dir: PathBuf,
    checksum_unpack_file: PathBuf,
}

#[async_trait]
impl Task for Unpack {
    fn description(&self) -> String {
        String::from("NetBird - Unpack")
    }
    async fn check_fulfilled(&self) -> Result<TaskFulfilled> {

        let unpacked_checksum_file = &self.checksum_unpack_file;
        if unpacked_checksum_file.exists() {
            let installed_digest = fs::read(unpacked_checksum_file)?;
            let distribution_digest = util::checksum::file(&self.from)?;

            if installed_digest == distribution_digest {
                return Ok(TaskFulfilled::Yes);
            }
        }
        Ok(TaskFulfilled::No)
    }
    async fn execute(&self) -> Result<Success> {
        let archive = File::open(&self.from)
            .context(format!("Failed to open NetBird distribution file at '{}'.", self.from.display()))?;
        let archive = GzDecoder::new(archive);
        let mut archive = tar::Archive::new(archive);
        archive.set_preserve_permissions(true);

        fs::create_dir_all(&self.to_dir)?;
        archive.unpack(&self.to_dir)?;

        let checksum = util::checksum::file(&self.from)?;
        let checksum_unpack_file = &self.checksum_unpack_file;
        fs::create_dir_all(checksum_unpack_file.parent().unwrap())?;
        fs::write(checksum_unpack_file, checksum)
            .context(format!("Writing checksum for netbird.tar.gz to '{}'.", checksum_unpack_file.display()))?;

        Ok(Success::default())
    }
}
impl Default for Unpack {
    fn default() -> Self {
        Self {
            from: constants::netbird::path_in_edgar_distribution().unwrap(),
            to_dir: constants::netbird::unpack_dir().unwrap(),
            checksum_unpack_file: constants::netbird::default_checksum_unpack_file()
        }
    }
}

#[cfg(test)]
mod tests {
    use assert_fs::prelude::*;
    use assert_fs::TempDir;
    
    use crate::common::task::{Task, TaskFulfilled};
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
            from: from.to_path_buf(),
            to_dir: to_dir.to_path_buf(),
            checksum_unpack_file: checksum_file_path.to_path_buf(),
        };

        assert_eq!(task.check_fulfilled().await?, TaskFulfilled::No);
        
        checksum_file_path.write_binary(&util::checksum::file(from.path())?)?;

        let task = Unpack {
            from: from.to_path_buf(),
            to_dir: to_dir.to_path_buf(),
            checksum_unpack_file: checksum_file_path.to_path_buf(),
        };

        assert_eq!(task.check_fulfilled().await?, TaskFulfilled::Yes);

        Ok(())
    }
}
