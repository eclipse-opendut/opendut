use std::fs;
use std::path::PathBuf;
use std::process::Command;

use anyhow::{Context, Result};

use crate::setup::constants::USER_NAME;
use crate::setup::task::{Success, Task, TaskFulfilled};
use crate::setup::util;
use crate::setup::util::EvaluateRequiringSuccess;

fn capability_file() -> PathBuf {
    PathBuf::from("/etc/security/capability.conf")
}
const LINE_TO_ADD: &str = "cap_net_admin opendut-network";

pub struct RequestCapabilityForUser;
impl Task for RequestCapabilityForUser {
    fn description(&self) -> String {
        String::from("Linux Network Capability - Request for User")
    }
    fn check_fulfilled(&self) -> Result<TaskFulfilled> {
        let capability_file = capability_file();

        if capability_file.exists() {
            let file_content = fs::read_to_string(&capability_file)
                .context(format!("Failed to read content of PAM file '{}'.", capability_file.display()))?;
            if file_content.contains(LINE_TO_ADD) {

                Command::new("su")
                    .arg(USER_NAME)
                    .arg("-c")
                    .arg("/sbin/capsh --has-i=cap_net_admin")
                    .evaluate_requiring_success()?;

                return Ok(TaskFulfilled::Yes)
            }
        }
        Ok(TaskFulfilled::No)
    }
    fn execute(&self) -> Result<Success> {
        util::prepend_line(LINE_TO_ADD, capability_file())?;
        Ok(Success::default())
    }
}
