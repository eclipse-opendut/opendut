use std::process::Command;

use anyhow::{Context, Result};
use crate::setup::constants::required_kernel_modules;

use crate::setup::task::{Success, Task, TaskFulfilled};

pub struct LoadKernelModules;
impl Task for LoadKernelModules {
    fn description(&self) -> String {
        let kernel_modules_str = required_kernel_modules().join(",");
        format!("Load Kernel Modules \"{kernel_modules_str}\"")
    }
    fn check_fulfilled(&self) -> Result<TaskFulfilled> {
        Ok(TaskFulfilled::Unchecked)
    }
    fn execute(&self) -> Result<Success> {

        for kernel_module in required_kernel_modules() {
            Command::new("modprobe")
                    .arg(kernel_module)
                    .status()
                    .context("Error when trying to load kernel module '{kernel_module}'.")?;
        }

        Ok(Success::default())
    }
}
