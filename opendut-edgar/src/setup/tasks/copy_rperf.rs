use std::fs;
use anyhow::Result;
use crate::setup::task::{Success, Task, TaskFulfilled};

pub struct CopyRperf;
impl Task for CopyRperf {
    fn description(&self) -> String {
        String::from("Copy the rperf distribution")
    }

    fn check_fulfilled(&self) -> Result<TaskFulfilled> {
        let rperf_path = crate::common::constants::rperf::executable_install_file();

        if rperf_path.exists() {
            Ok(TaskFulfilled::Yes)
        } else {
            Ok(TaskFulfilled::No)
        }
    }
    
    fn execute(&self) -> Result<Success> {
        let path_in_edgar_distribution = crate::setup::constants::rperf::path_in_edgar_distribution()?;
        let target_path = crate::common::constants::rperf::executable_install_file();

        fs::create_dir_all(target_path.parent().unwrap())?;
        fs::copy(path_in_edgar_distribution, target_path)?;

        Ok(Success::default())
    }
}
