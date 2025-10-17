use std::ops::Not;
use anyhow::{Context, Result};
use std::process::Command;
use async_trait::async_trait;
use crate::common::task::{Success, Task, TaskStateFulfilled};
use crate::setup::constants::{REQUIRED_COMMAND_LINE_PROGRAMS, REQUIRED_COMMAND_LINE_PROGRAMS_CAN};
use crate::setup::util::EvaluateRequiringSuccess;


pub struct CheckCommandLinePrograms {
    pub skip_can_setup: bool,
}

#[async_trait]
impl Task for CheckCommandLinePrograms {
    fn description(&self) -> String {
        String::from("Check availability of needed command line programs")
    }
    async fn check_present(&self) -> Result<TaskStateFulfilled> {
        Ok(TaskStateFulfilled::Unchecked)
    }
    async fn make_present(&self) -> Result<Success> {

        let required_programs = {
            let mut required_programs = vec![];
            required_programs.extend(REQUIRED_COMMAND_LINE_PROGRAMS);

            if self.skip_can_setup.not() {
                required_programs.extend(REQUIRED_COMMAND_LINE_PROGRAMS_CAN);
            }

            required_programs
        };

        for (command_line_program, arg) in required_programs.iter() {
            let executable = which::which(command_line_program)
                .context(format!(
                    "Command-line program `{command_line_program}` is required.\n\
                    It could not be found in the operating system PATH. Make sure, it is installed correctly"
                ))?;

            let mut command = Command::new(executable);
            command.arg(arg);
            command
                .evaluate_requiring_success()
                .context(format!(
                    "Command-line program `{command_line_program}` was found in the operating system PATH,\n\
                    but running a basic command ({command:?}) with it, did not succeed"
                ))?;
        }
        
        Ok(Success::default())
    }
}
