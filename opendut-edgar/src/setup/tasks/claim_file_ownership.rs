use anyhow::{bail, Result};
use async_trait::async_trait;
use crate::common::settings::default_config_file_path;
use crate::common::task::{Success, Task, TaskFulfilled};
use crate::setup::User;
use crate::setup::util::{chown, create_file_and_ensure_it_can_only_be_read_or_modified_by_owner};

const DIRS: &[&str] = &[
    "/opt/opendut/",
    "/etc/opendut/",
];

pub struct ClaimFileOwnership {
    pub service_user: User,
}

#[async_trait]
impl Task for ClaimFileOwnership {
    fn description(&self) -> String {
        String::from("Claim File Ownership")
    }
    async fn check_fulfilled(&self) -> Result<TaskFulfilled> {
        Ok(TaskFulfilled::Unchecked)
    }
    async fn execute(&self) -> Result<Success> {
        for dir in DIRS {
            for path_result in walkdir::WalkDir::new(dir) {
                match path_result {
                    Ok(path) => chown(&self.service_user, path.path())?,
                    Err(cause) => bail!("Error while setting ownership for a sub-path in directory '{dir}': {cause}"),
                }
            }
        }
        let edgar_toml = default_config_file_path();
        create_file_and_ensure_it_can_only_be_read_or_modified_by_owner(&edgar_toml)?;
        Ok(Success::default())
    }
}
