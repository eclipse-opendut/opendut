use anyhow::Result;
use crate::setup::constants::required_kernel_modules;

use crate::setup::task::{Success, Task, TaskFulfilled};

pub struct LoadKernelModules;
impl Task for LoadKernelModules {
    fn description(&self) -> String {
        let kernel_modules_str = required_kernel_modules().into_iter().map(|m| m.name).collect::<Vec<String>>().join(", ");

        format!("Load Kernel Modules \"{kernel_modules_str}\"")
    }
    fn check_fulfilled(&self) -> Result<TaskFulfilled> {
        for kernel_module in required_kernel_modules() {
            if ! kernel_module.is_loaded()? {
                return Ok(TaskFulfilled::No)
            }
        }
        Ok(TaskFulfilled::Yes)
    }
    fn execute(&self) -> Result<Success> {

        for kernel_module in required_kernel_modules() {
            kernel_module.load()?;
        }

        Ok(Success::default())
    }
}
