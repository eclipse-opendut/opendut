use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::setup::task::{Success, Task, TaskFulfilled};
use crate::setup::util;

const LINE_TO_ADD: &str = "auth optional pam_cap.so";

pub struct MakePamAuthOptional {
    pam_file: PathBuf,
}
impl Task for MakePamAuthOptional {
    fn description(&self) -> String {
        String::from("Linux Network Capability - Make PAM Authentication Optional")
    }
    fn check_fulfilled(&self) -> Result<TaskFulfilled> {
        if self.pam_file.exists() {
            let file_content = fs::read_to_string(&self.pam_file)
                .context(format!("Failed to read content of PAM file '{}'.", self.pam_file.display()))?;
            if file_content.contains(LINE_TO_ADD) {
                return Ok(TaskFulfilled::Yes)
            }
        }
        Ok(TaskFulfilled::No)
    }
    fn execute(&self) -> Result<Success> {
        util::prepend_line(LINE_TO_ADD, &self.pam_file)?;
        Ok(Success::default())
    }
}
impl Default for MakePamAuthOptional {
    fn default() -> Self {
        Self {
            pam_file: PathBuf::from("/etc/pam.d/su"),
        }
    }
}

#[cfg(test)]
mod tests {
    use assert_fs::prelude::*;
    use assert_fs::TempDir;
    use crate::setup::runner;

    use super::*;

    #[test]
    fn happy_flow() -> Result<()> {
        let temp = TempDir::new()?;
        let pam_file = temp.child("pam_file");

        const OTHER_TEXT: &str = "OTHER_TEXT";
        pam_file.write_str(OTHER_TEXT)?;

        let task = MakePamAuthOptional {
            pam_file: pam_file.to_path_buf(),
        };

        runner::test::no_then_yes(task)?;

        Ok(())
    }
}
