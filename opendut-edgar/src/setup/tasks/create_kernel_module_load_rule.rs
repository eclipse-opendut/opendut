use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use opendut_edgar_kernel_modules::{required_kernel_modules, KernelModule};

use crate::setup::task::{Success, Task, TaskFulfilled};
use crate::setup::constants::KERNEL_MODULE_LOAD_RULE_PREFIX;

// Returns the file path for the configuation file that causes the kernel module to be loaded during boot
fn load_rule_file_path(kernel_module: &KernelModule) -> PathBuf {
    PathBuf::from(format!("/etc/modules-load.d/{KERNEL_MODULE_LOAD_RULE_PREFIX}-{}.conf", kernel_module.name))
}

// Returns the file path for the configuation file that causes the kernel modules loaded at boot time to be loaded with the correct parameters
fn options_rule_file_path(kernel_module: &KernelModule) -> PathBuf {
    PathBuf::from(format!("/etc/modprobe.d/{KERNEL_MODULE_LOAD_RULE_PREFIX}-{}.conf", kernel_module.name))
}

fn load_rule_file_content(kernel_module: &KernelModule) -> String {
    format!("{}\n", kernel_module.name)
}

fn options_rule_file_content(kernel_module: &KernelModule) -> String {
    let mut rule = format!("options {}", kernel_module.name);

    for (key, value) in &kernel_module.params{
        rule.push_str(format!(" {key}={value}").as_str());
    }

    rule.push('\n');

    rule

}

pub struct CreateKernelModuleLoadRule;
impl Task for CreateKernelModuleLoadRule {
    fn description(&self) -> String {
        let kernel_modules_str = required_kernel_modules().into_iter().map(|m| m.name).collect::<Vec<String>>().join(", ");

        format!("Create rules to load kernel modules {kernel_modules_str} at boot time")
    }
    fn check_fulfilled(&self) -> Result<TaskFulfilled> {
        for kernel_module in required_kernel_modules() {
            if !load_rule_file_path(&kernel_module).exists() {
                return Ok(TaskFulfilled::No);
            }
            if !kernel_module.params.is_empty() && !options_rule_file_path(&kernel_module).exists() {
                return Ok(TaskFulfilled::No);
            }
        }
        Ok(TaskFulfilled::Yes)
    }
    fn execute(&self) -> Result<Success> {
        for kernel_module in required_kernel_modules() {
            let load_path = load_rule_file_path(&kernel_module);
            fs::create_dir_all(load_path.parent().unwrap())?;

            fs::write(&load_path, load_rule_file_content(&kernel_module))
                .context(format!("Error while writing kernel module load file '{}'", load_path.display()))?;

            if !kernel_module.params.is_empty() {
                let options_path = options_rule_file_path(&kernel_module);
                fs::create_dir_all(options_path.parent().unwrap())?;

                fs::write(&options_path, options_rule_file_content(&kernel_module))
                    .context(format!("Error while writing kernel module parameter file '{}'", options_path.display()))?;
            }
        }

        Ok(Success::default())
    }
}
