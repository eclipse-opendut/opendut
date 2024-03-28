use std::path::PathBuf;
use anyhow::{Result, bail};

use crate::setup::util::running_in_docker;
use crate::setup::task::{Success, Task, TaskFulfilled};

pub struct LoadKernelModules{
    loaded_module_file: PathBuf,
    builtin_module_dir: PathBuf,
}

impl Task for LoadKernelModules {
    fn description(&self) -> String {
        let kernel_modules_str = opendut_edgar_kernel_modules::required_kernel_modules()
            .into_iter()
            .map(|m| m.name)
            .collect::<Vec<String>>()
            .join(", ");

        format!("Load Kernel Modules {kernel_modules_str}")
    }
    fn check_fulfilled(&self) -> Result<TaskFulfilled> {
        for kernel_module in opendut_edgar_kernel_modules::required_kernel_modules() {
            if ! kernel_module.is_loaded(&self.loaded_module_file, &self.builtin_module_dir)? {
                return Ok(TaskFulfilled::No)
            }
        }
        Ok(TaskFulfilled::Yes)
    }
    fn execute(&self) -> Result<Success> {
        if running_in_docker() {
            bail!("Cannot load kernel modules from within Docker. Modules must be loaded from the host.");
        }

        for kernel_module in opendut_edgar_kernel_modules::required_kernel_modules() {
            kernel_module.load()?;
        }

        Ok(Success::default())
    }
}

impl Default for LoadKernelModules {
    fn default() -> Self {
        Self {
            loaded_module_file: opendut_edgar_kernel_modules::default_module_file(),
            builtin_module_dir: opendut_edgar_kernel_modules::default_builtin_module_dir(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use assert_fs::fixture::ChildPath;
    use assert_fs::prelude::*;
    use assert_fs::TempDir;
    use indoc::indoc;
    use rstest::{fixture, rstest};
    use crate::setup::task::{Task, TaskFulfilled};
    use crate::setup::tasks::LoadKernelModules;

    #[rstest]
    fn should_check_task_is_fulfilled_for_loaded_kernel_modules(fixture: Fixture) -> anyhow::Result<()> {
        fixture.loaded_module_file.write_str(indoc!(r"
            bridge 413696 1 br_netfilter, Live 0x0000000000000000
        "))?;
        
        let task = LoadKernelModules {
            loaded_module_file: fixture.loaded_module_file.to_path_buf(),
            builtin_module_dir: fixture.builtin_module_dir.to_path_buf(),
        };

        assert_eq!(task.check_fulfilled()?, TaskFulfilled::No);

        fixture.loaded_module_file.write_str(indoc!(r"
            vcan 12288 0 - Live 0x0000000000000000
            bridge 413696 1 br_netfilter, Live 0x0000000000000000
            can_gw 32768 0 - Live 0x0000000000000000
        "))?;

        assert_eq!(task.check_fulfilled()?, TaskFulfilled::Yes);
        Ok(())
    }

    #[rstest]
    fn should_check_task_is_fulfilled_for_builtin_kernel_modules(fixture: Fixture) -> anyhow::Result<()> {
        let bridge_dir = fixture.builtin_module_dir.child("bridge");
        fs::create_dir_all(bridge_dir)?;
        
        let task = LoadKernelModules {
            loaded_module_file: fixture.loaded_module_file.to_path_buf(),
            builtin_module_dir: fixture.builtin_module_dir.to_path_buf(),
        };

        assert_eq!(task.check_fulfilled()?, TaskFulfilled::No);
        
        for module in ["vcan", "can_gw"] {
            let module_dir = fixture.builtin_module_dir.child(module);
            fs::create_dir_all(&module_dir)?;
        }
        
        assert_eq!(task.check_fulfilled()?, TaskFulfilled::Yes);
        Ok(())
    }
    
    struct Fixture {
        _temp: TempDir,
        loaded_module_file: ChildPath,
        builtin_module_dir: ChildPath,
    }
    #[fixture]
    fn fixture() -> Fixture {
        let temp = TempDir::new().unwrap();
        
        let loaded_module_file = temp.child("testmodule");
        loaded_module_file.touch().unwrap();
        
        let builtin_module_dir = temp.child("builtin");
        fs::create_dir_all(&builtin_module_dir).unwrap();
        
        Fixture {
            _temp: temp,
            loaded_module_file,
            builtin_module_dir
        }
    }
}
