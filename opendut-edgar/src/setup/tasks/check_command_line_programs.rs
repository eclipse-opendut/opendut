use anyhow::{Context, Result};
use std::process::Command;

use crate::setup::task::{Success, Task, TaskFulfilled};
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

        for (command_line_program, arg) in REQUIRED_COMMAND_LINE_PROGRAMS.iter(){
            Command::new(command_line_program)
            .arg(arg)
            .evaluate_requiring_success()
            .context(format!("{command_line_program} is required."))?;
        }
        
        Ok(Success::default())
    }
}
