use anyhow::Error;
use opendut_edgar_kernel_modules::{default_builtin_module_dir, default_module_file, required_can_kernel_modules};

pub fn load_linux_kernel_modules_for_can() -> Result<(), Error> {
    for kernel_module in required_can_kernel_modules() {
        if !kernel_module.is_loaded(&default_module_file(), &default_builtin_module_dir())? {
            kernel_module.load()?;
        }
    }
    Ok(())
}