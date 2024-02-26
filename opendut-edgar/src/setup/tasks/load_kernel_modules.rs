use anyhow::{Result, Error};

use opendut_edgar_kernel_modules::edgar_required_kernel_modules;

use crate::setup::util::running_in_docker;
use crate::setup::task::{Success, Task, TaskFulfilled};

pub struct LoadKernelModules;
impl Task for LoadKernelModules {
    fn description(&self) -> String {
        let kernel_modules_str = edgar_required_kernel_modules().into_iter().map(|m| m.name).collect::<Vec<String>>().join(", ");

        format!("Load Kernel Modules \"{kernel_modules_str}\"")
    }
    fn check_fulfilled(&self) -> Result<TaskFulfilled> {
        for kernel_module in edgar_required_kernel_modules() {
            if ! kernel_module.is_loaded()? {
                return Ok(TaskFulfilled::No)
            }
        }
        Ok(TaskFulfilled::Yes)
    }
    fn execute(&self) -> Result<Success> {
        if running_in_docker() {
            return Err(Error::msg("Cannot load kernel modules from within Docker. Modules must be loaded from the host."))
        }

        for kernel_module in edgar_required_kernel_modules() {
            kernel_module.load()?;
        }

        Ok(Success::default())
    }
}
