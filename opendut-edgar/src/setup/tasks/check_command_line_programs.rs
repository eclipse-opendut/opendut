use anyhow::{Context, Result};
use std::process::Command;

use crate::common::task::{Success, Task, TaskFulfilled};
use crate::setup::constants::REQUIRED_COMMAND_LINE_PROGRAMS;
use crate::setup::util::EvaluateRequiringSuccess;


pub struct CheckCommandLinePrograms;
impl Task for CheckCommandLinePrograms {
    fn description(&self) -> String {
        String::from("Check availabilty of needed command line programs")
    }
    fn check_fulfilled(&self) -> Result<TaskFulfilled> {
        Ok(TaskFulfilled::Unchecked)
    }
    fn execute(&self) -> Result<Success> {

        for (command_line_program, arg) in REQUIRED_COMMAND_LINE_PROGRAMS.iter() {
            let executable = which::which(command_line_program)
                .context(format!(
                    "Command-line program `{command_line_program}` is required.\n\
                    It could not be found in the operating system PATH. Make sure, it is installed correctly."
                ))?;

            let mut command = Command::new(executable);
            command.arg(arg);
            command
                .evaluate_requiring_success()
                .context(format!(
                    "Command-line program `{command_line_program}` was found in the operating system PATH,\n\
                    but running a basic command ({command:?}) with it, did not succeed."
                ))?;
        }
        
        Ok(Success::default())
    }
}
