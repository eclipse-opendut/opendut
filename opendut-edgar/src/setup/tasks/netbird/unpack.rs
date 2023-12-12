use std::fs;
use std::fs::File;
use std::path::PathBuf;

use anyhow::{Context, Result};
use flate2::read::GzDecoder;

use crate::setup::{constants, util};
use crate::setup::task::{Success, Task, TaskFulfilled};

pub struct Unpack {
    from: PathBuf,
    to_dir: PathBuf,
}
impl Task for Unpack {
    fn description(&self) -> String {
        String::from("NetBird - Unpack")
    }
    fn check_fulfilled(&self) -> Result<TaskFulfilled> {

        let unpacked_checksum_file = constants::netbird::checksum_unpack_file();
        if unpacked_checksum_file.exists() {
            let installed_digest = fs::read(unpacked_checksum_file)?;
            let distribution_digest = util::file_checksum(&self.from)?;

            if installed_digest == distribution_digest {
                return Ok(TaskFulfilled::Yes);
            }
        }
        Ok(TaskFulfilled::No)
    }
    fn execute(&self) -> Result<Success> {
        let archive = File::open(&self.from)
            .context(format!("Failed to open NetBird distribution file at '{}'.", self.from.display()))?;
        let archive = GzDecoder::new(archive);
        let mut archive = tar::Archive::new(archive);
        archive.set_preserve_permissions(true);

        fs::create_dir_all(&self.to_dir)?;
        archive.unpack(&self.to_dir)?;

        let checksum = util::file_checksum(&self.from)?;
        let checksum_unpack_file = constants::netbird::checksum_unpack_file();
        fs::create_dir_all(checksum_unpack_file.parent().unwrap())?;
        fs::write(&checksum_unpack_file, checksum)
            .context(format!("Writing checksum for netbird.tar.gz to '{}'.", checksum_unpack_file.display()))?;

        Ok(Success::default())
    }
}
impl Default for Unpack {
    fn default() -> Self {
        Self {
            from: constants::netbird::path_in_edgar_distribution().unwrap(),
            to_dir: constants::netbird::unpack_dir().unwrap(),
        }
    }
}
