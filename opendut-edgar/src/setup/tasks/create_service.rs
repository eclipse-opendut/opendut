use std::fs;
use std::path::PathBuf;
use std::process::Command;

use anyhow::{Context, Result};

use crate::setup::task::{Success, Task, TaskFulfilled};
use crate::setup::constants::{executable_install_path, USER_NAME};
use crate::setup::constants::SYSTEMD_SERVICE_FILE_NAME;
use crate::setup::util::EvaluateRequiringSuccess;


pub fn systemd_file_path() -> PathBuf {
    PathBuf::from(format!("/etc/systemd/system/{SYSTEMD_SERVICE_FILE_NAME}"))
}

fn systemd_file_content() -> String {
    let executable = executable_install_path().unwrap();
    let executable = executable.display();

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
User={USER_NAME}
Group={USER_NAME}

[Install]
WantedBy=multi-user.target
"#)
// backoff strategy as soon as https://github.com/systemd/systemd/pull/26902 is released in ubuntu (systemd v254 Latest)
// [Service]
// RestartSec=3s
// RestartMaxDelaySec=30s
// RestartSteps=5
}

pub struct CreateServiceFile;
impl Task for CreateServiceFile {
    fn description(&self) -> String {
        String::from("Create Service File")
    }
    fn check_fulfilled(&self) -> Result<TaskFulfilled> {
        if systemd_file_path().exists() {
            Ok(TaskFulfilled::Yes)
        } else {
            Ok(TaskFulfilled::No)
        }
    }
    fn execute(&self) -> Result<Success> {
        let out_path = systemd_file_path();
        fs::create_dir_all(out_path.parent().unwrap())?;

        fs::write(&out_path, systemd_file_content())
            .context(format!("Error while writing service file to '{}'", out_path.display()))?;

        let _ = Command::new("systemctl")
            .arg("daemon-reload")
            .evaluate_requiring_success()?;

        Ok(Success::default())
    }
}
