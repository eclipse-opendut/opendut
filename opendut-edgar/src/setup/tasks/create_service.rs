use std::fs;
use std::path::PathBuf;
use std::process::Command;

use anyhow::{Context, Result};

use crate::setup::task::{Success, Task, TaskFulfilled};
use crate::setup::constants::executable_install_path;
use crate::setup::constants::SYSTEMD_SERVICE_FILE_NAME;
use crate::setup::{User, util};
use crate::setup::util::{CommandRunner, DefaultCommandRunner};

pub fn default_systemd_file_path() -> PathBuf {
    PathBuf::from(format!("/etc/systemd/system/{SYSTEMD_SERVICE_FILE_NAME}"))
}

pub fn default_checksum_systemd_file_path() -> PathBuf {
    PathBuf::from("/etc/systemd/system/.opendut-edgar.service.checksum")
}

fn systemd_file_content(service_user: &User) -> String {
    let executable = executable_install_path().unwrap();
    let executable = executable.display();
    let service_user = &service_user.name;

    format!(r#"
[Unit]
Description=openDuT EDGAR
Requires=network-online.target
After=network-online.service
StartLimitIntervalSec=5
StartLimitBurst=0

[Service]
ExecStart={executable} service
Restart=always
RestartSec=30s
User={service_user}
Group={service_user}


[Install]
WantedBy=multi-user.target
"#)
// backoff strategy as soon as https://github.com/systemd/systemd/pull/26902 is released in ubuntu (systemd v254 Latest)
// [Service]
// RestartSec=3s
// RestartMaxDelaySec=30s
// RestartSteps=5
}

pub struct CreateServiceFile {
    pub service_user: User,
    pub systemd_file_path: PathBuf,
    pub checksum_systemd_file: PathBuf,
    pub command_runner: Box<dyn CommandRunner>,
}
impl Task for CreateServiceFile {
    fn description(&self) -> String {
        String::from("Create Service File")
    }
    fn check_fulfilled(&self) -> Result<TaskFulfilled> {
        let unpacked_systemd_checksum_file = &self.checksum_systemd_file;
        if unpacked_systemd_checksum_file.exists() {
            let systemd_installed_digest = fs::read(unpacked_systemd_checksum_file)?;
            let systemd_distribution_digest = util::checksum::string(systemd_file_content(&self.service_user))?;

            if systemd_installed_digest == systemd_distribution_digest {
                return Ok(TaskFulfilled::Yes);
            }
        }
        Ok(TaskFulfilled::No)
    }
    fn execute(&self) -> Result<Success> {
        let out_path = &self.systemd_file_path;
        fs::create_dir_all(out_path.parent().unwrap())?;
        
        let systemd_file_content = systemd_file_content(&self.service_user);

        fs::write(out_path, &systemd_file_content)
            .context(format!("Error while writing service file to '{}'", out_path.display()))?;

        self.command_runner.run(
            Command::new("systemctl").arg("daemon-reload")
        ).context("systemctl daemon-reload could not be executed successfully!")?;

        let checksum_systemd_file = util::checksum::string(systemd_file_content)?;
        let checksum_systemd_file_unpack_file = &self.checksum_systemd_file;
        fs::create_dir_all(checksum_systemd_file_unpack_file.parent().unwrap())?;
        fs::write(checksum_systemd_file_unpack_file, checksum_systemd_file)
            .context(format!("Writing checksum for opendut-edgar.service to '{}'.", checksum_systemd_file_unpack_file.display()))?;

        Ok(Success::default())
    }
}

impl CreateServiceFile {
    pub fn with_service_user(service_user: User) -> Self {
        Self {
            service_user,
            systemd_file_path: default_systemd_file_path(),
            checksum_systemd_file: default_checksum_systemd_file_path(),
            command_runner: Box::new(DefaultCommandRunner),
        }
    }
}

#[cfg(test)]
mod tests {
    use assert_fs::prelude::*;
    use assert_fs::TempDir;

    use crate::setup::constants::SYSTEMD_SERVICE_FILE_NAME;
    use crate::setup::task::{Task, TaskFulfilled};
    use crate::setup::tasks::CreateServiceFile;
    use crate::setup::User;
    use crate::setup::util::NoopCommandRunner;

    #[test]
   fn should_check_task_is_fulfilled() -> anyhow::Result<()> {
        let temp = TempDir::new().unwrap();

        let systemd_file_path = temp.child(SYSTEMD_SERVICE_FILE_NAME);

        let checksum_systemd_file_path = temp.child(format!("{SYSTEMD_SERVICE_FILE_NAME}.checksum"));

        let task = CreateServiceFile {
            service_user: User { name: "testUser".to_string() },
            systemd_file_path: systemd_file_path.to_path_buf(),
            checksum_systemd_file: checksum_systemd_file_path.to_path_buf(),
            command_runner: Box::new(NoopCommandRunner),
        };

        assert_eq!(task.check_fulfilled()?, TaskFulfilled::No);
        task.execute()?;
        assert_eq!(task.check_fulfilled()?, TaskFulfilled::Yes);

        Ok(())
    }
}
