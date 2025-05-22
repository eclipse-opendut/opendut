use crate::fs;
use anyhow::Result;
use async_trait::async_trait;
use crate::common::task::{Success, Task, TaskStateFulfilled};

pub struct CopyRperf;

#[async_trait]
impl Task for CopyRperf {
    fn description(&self) -> String {
        String::from("Copy the rperf distribution")
    }

    async fn check_present(&self) -> Result<TaskStateFulfilled> {
        let rperf_path = crate::common::constants::rperf::executable_install_file();

        if rperf_path.exists() {
            Ok(TaskStateFulfilled::Yes)
        } else {
            Ok(TaskStateFulfilled::No)
        }
    }
    
    async fn make_present(&self) -> Result<Success> {
        let path_in_edgar_distribution = crate::setup::constants::rperf::path_in_edgar_distribution()?;
        let target_path = crate::common::constants::rperf::executable_install_file();

        fs::create_dir_all(target_path.parent().unwrap())?;
        fs::copy(path_in_edgar_distribution, target_path)?;

        Ok(Success::default())
    }
}
