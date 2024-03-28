use std::collections::HashMap;
use std::process::Command;
use std::fs::{File};
use std::io::{self, prelude::*, BufReader};
use tracing::debug;
use std::path::{Path, PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failure while checking for loaded module: {cause}")]
    CheckModuleLoaded { cause: io::Error },
    #[error("Failure while loading module: {cause}")]
    LoadModule { cause: io::Error },
    #[error("Failure while loading module: {cause}")]
    LoadModuleExecution { cause: String },
}

pub struct KernelModule {
    pub name: String,
    pub params: HashMap<String, String>
}

impl KernelModule {
    pub fn is_loaded(&self, loaded_module_file: &Path, builtin_module_dir: &Path) -> Result<bool, Error> {
        let module = str::replace(self.name.as_str(), "-", "_");
        
        let file = File::open(loaded_module_file)
            .map_err(|cause| Error::CheckModuleLoaded { cause })?;
        let reader = BufReader::new(file);

        // TODO: Should not only check that module is loaded but also that it's loaded with the correct options
        for mod_line in reader.lines() {
            match mod_line {
                Ok(line) => {
                    match line.split(' ').collect::<Vec<&str>>().first() {
                        Some(mod_name) => {
                            if str::replace(mod_name, "-", "_") == module {
                                return Ok(true)
                            }
                        }
                        None => continue
                    }
                }
                Err(why) => return Err(Error::CheckModuleLoaded { cause: why }),
            }
        }
        Ok(builtin_module_dir.join(module).exists())
    }

    pub fn load(&self) -> Result<(), Error> {

        let mut cmd =
            if nix::unistd::geteuid() == nix::unistd::ROOT {
                Command::new("modprobe")
            } else {
                debug!("Not running with effective UID of root. Using sudo to run `modprobe`.");
                let mut cmd = Command::new("sudo");
                cmd.arg("modprobe");
                cmd
            };
        cmd.arg(&self.name);

        for (key, value) in &self.params {
            cmd.arg(format!("{key}={value}"));
        }

        let output = cmd.output().map_err(|cause| Error::LoadModule { cause })?;

        if ! output.status.success() {
            return Err(Error::LoadModuleExecution { cause: format!("{:?}", String::from_utf8_lossy(&output.stderr).trim()) });
        }
        Ok(())
    }
}


pub fn required_kernel_modules() -> Vec<KernelModule> {
    vec![
        KernelModule {
            name: "vcan".to_string(),
            params: HashMap::new(),
        },
        KernelModule {
            name: "can-gw".to_string(),
            params: HashMap::from([
                ("max_hops".to_string(), "2".to_string())
            ]),
        }
    ]
}

pub fn default_module_file() -> PathBuf {
    PathBuf::from("/proc/modules")
}

pub fn default_builtin_module_dir() -> PathBuf {
    PathBuf::from("/sys/module")
}
