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
    #[error("Failure while checking module <{module}> parameter <{parameter}>: {cause}")]
    CheckModuleParameters { module: String, parameter: String, cause: io::Error },
    #[error("Invalid parameters for module <{module}> with parameter <{parameter}>: {cause}")]
    InvalidModuleParameters { module: String, parameter: String, cause: String },
    #[error("Failure while loading module: {cause}")]
    LoadModule { cause: io::Error },
    #[error("Failure while loading module: {cause}")]
    LoadModuleExecution { cause: String },
}

pub struct KernelModule {
    name: String,
    pub params: HashMap<String, String>
}

pub struct KernelParameterFile(PathBuf);
impl KernelParameterFile {
    pub fn new(module_dir: &Path, module: &str, parameter: &str) -> Self {
        Self(module_dir.join(module).join("parameters").join(parameter))
    }

    pub fn value(&self) -> &PathBuf {
        &self.0
    }
}


impl KernelModule {
    pub fn is_loaded(&self, loaded_module_file: &Path, builtin_module_dir: &Path) -> Result<bool, Error> {
        let file = File::open(loaded_module_file)
            .map_err(|cause| Error::CheckModuleLoaded { cause })?;
        let reader = BufReader::new(file);

        for mod_line in reader.lines() {
            match mod_line {
                Ok(line) => {
                    match line.split(' ').collect::<Vec<&str>>().first() {
                        Some(mod_name) => {
                            if str::replace(mod_name, "-", "_") == self.name() {
                                return Ok(true)
                            }
                        }
                        None => continue
                    }
                }
                Err(why) => return Err(Error::CheckModuleLoaded { cause: why }),
            }
        }
        self.check_module_parameters(builtin_module_dir)?;

        Ok(builtin_module_dir.join(self.name()).exists())
    }

    /// Generic module name with hyphens replaced by underscores
    pub fn name(&self) -> String {
        str::replace(self.name.as_str(), "-", "_")
    }

    fn check_module_parameters(&self, module_dir: &Path) -> Result<(), Error> {
        let module = self.name();
        for (parameter, value) in &self.params {
            let param_file = KernelParameterFile::new(module_dir, &module, parameter);
            let mut file = File::open(param_file.value())
                .map_err(|cause| Error::CheckModuleParameters { module: module.clone(), parameter: parameter.clone(), cause })?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)
                .map_err(|cause| Error::CheckModuleParameters { module: module.clone(), parameter: parameter.clone(), cause })?;
            if contents.trim() != value.as_str() {
                return Err(Error::InvalidModuleParameters { module, parameter: parameter.clone(), cause: contents.trim().to_string() });
            }
        }
        Ok(())
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


pub fn required_can_kernel_modules() -> Vec<KernelModule> {
    vec![
        KernelModule {
            name: "vcan".to_string(),
            params: HashMap::new(),
        },
        KernelModule {
            name: "can_gw".to_string(),
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
