use anyhow::{bail, Result};

use crate::setup::task::{Success, Task, TaskFulfilled};
use crate::setup::User;
use crate::setup::util::chown;

const DIRS: &[&str] = &[
    "/opt/opendut/",
    "/etc/opendut/",
];

pub struct ClaimFileOwnership {
    pub service_user: User,
}
impl Task for ClaimFileOwnership {
    fn description(&self) -> String {
        String::from("Claim File Ownership")
    }
    fn check_fulfilled(&self) -> Result<TaskFulfilled> {
        Ok(TaskFulfilled::Unchecked)
    }
    fn execute(&self) -> Result<Success> {
        for dir in DIRS {
            for path_result in walkdir::WalkDir::new(dir) {
                match path_result {
                    Ok(path) => chown(&self.service_user, path.path())?,
                    Err(cause) => bail!("Error while setting ownership for a sub-path in directory '{dir}': {cause}"),
                }
            }
        }
        Ok(Success::default())
    }
}
