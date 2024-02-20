use anyhow::Result;
use opendut_util::kernel_modules::edgar_required_kernel_modules;

use crate::setup::task::{Success, Task, TaskFulfilled};
use anyhow::Error;

pub struct CheckKernelModulesLoaded;
impl Task for CheckKernelModulesLoaded {
    fn description(&self) -> String {
        let kernel_modules_str = edgar_required_kernel_modules().into_iter().map(|m| m.name).collect::<Vec<String>>().join(", ");

        format!("Check if the required kernel modules are loaded: {kernel_modules_str}")
    }
    fn check_fulfilled(&self) -> Result<TaskFulfilled> {
        
        Ok(TaskFulfilled::Unchecked)
    }
    fn execute(&self) -> Result<Success> {

        for kernel_module in edgar_required_kernel_modules() {
            if ! kernel_module.is_loaded()? {
                return Err(Error::msg(format!("Kernel module {} is not loaded", kernel_module.name)))
            }
        }

        Ok(Success::default())
    }
}
