use tracing::{debug, error, info};

use crate::common::task::{Success, Task, TaskStateFulfilled};
use crate::common::task::progress_bar::ProgressBarForCLI;

pub async fn run(run_mode: RunMode, tasks: &[Box<dyn Task>]) -> Result<(), TaskExecutionError> {
    if tasks.is_empty() {
        debug!("No tasks to run. Skipping.");
        return Ok(())
    }

    let task_names_string = tasks.iter().map(|task| task.description()).collect::<Vec<_>>().join(", ");
    debug!("Running tasks: {task_names_string}");

    if run_mode != RunMode::Service {
        println!();
    }
    
    run_tasks(tasks, run_mode).await?;

    if run_mode != RunMode::Service {
        println!();
    }

    debug!("Completed running tasks: {task_names_string}");
    Ok(())
}

async fn run_tasks(
    tasks: &[Box<dyn Task>],
    run_mode: RunMode,
) -> Result<(), TaskExecutionError> {
    let progress_style = ProgressBarForCLI::progress_style();
    for task in tasks {
        let progress_bar_cli = ProgressBarForCLI::new(task.description(), run_mode, progress_style.clone());
        let is_fulfilled = task.check_present()
            .await
            .map_err(|error| TaskExecutionError::DetermineSystemStateBefore { task_name: task.description(), error })?;

        let outcome = match is_fulfilled {
            TaskStateFulfilled::Yes => Outcome::Unchanged,
            TaskStateFulfilled::No | TaskStateFulfilled::Unchecked => {
                if run_mode == RunMode::SetupDryRun {
                    Outcome::DryRun
                } else {
                    let result = task.make_present()
                        .await;
                    progress_bar_cli.finish_and_clear();
                    match result {
                        Ok(success) => Outcome::Changed(success),
                        Err(error) => {
                            return Err(TaskExecutionError::DuringTaskExecution { task_name: task.description(), error });
                        }
                    }
                }
            }
        };
        progress_bar_cli.finish_and_clear();

        if let Outcome::Changed(_) = outcome {
            match task.check_present().await {
                Ok(fulfillment) => match fulfillment {
                    TaskStateFulfilled::Yes | TaskStateFulfilled::Unchecked => {}, //do nothing
                    TaskStateFulfilled::No => {
                        return Err(TaskExecutionError::UnfulfilledTask { task_name: task.description() });
                    }
                }
                Err(error) => {
                    return Err(TaskExecutionError::DetermineSystemStateAfter { task_name: task.description(), error });
                }
            }
        };
        if run_mode != RunMode::Service {
            print_outcome(task.description(), outcome)
        }
    }
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum TaskExecutionError {
    #[error("Error while determining system state: {error}")]
    DetermineSystemStateBefore { task_name: String, error: anyhow::Error },
    #[error("Error while executing a task: {error}")]
    DuringTaskExecution { task_name: String, error: anyhow::Error },
    #[error("Error while executing a task.")]
    UnfulfilledTask { task_name: String },
    #[error("Error while determining system state after execution: {error}")]
    DetermineSystemStateAfter { task_name: String, error: anyhow::Error },
}

impl TaskExecutionError {
    pub fn print_error(&self) {
        let (task_name, error) = match self {
            TaskExecutionError::DetermineSystemStateBefore { task_name, error } => {
                (task_name, Some(error))
            }
            TaskExecutionError::DuringTaskExecution { task_name, error } => {
                (task_name, Some(error))
            }
            TaskExecutionError::UnfulfilledTask { task_name } => {
                (task_name, None)
            }
            TaskExecutionError::DetermineSystemStateAfter { task_name, error } => {
                (task_name, Some(error))
            }
        };

        let message = {
            let mut message = String::new();
            message.push_str(task_name);
            message.push('\n');

            if let Some(error) = error {
                let error = format!("{:#}", error);
                for line in error.lines() {
                    message.push_str(line);
                    message.push('\n');
                }
            }
            message
        };
        for line in message.lines() {
            eprintln!("    {}", line);
        }
        error!("{message}");
        print_outcome(task_name.to_string(), Outcome::Failed);
    }
}


#[derive(Clone, Copy, PartialEq)]
pub enum RunMode { Setup, SetupDryRun, Service }

#[derive(Debug)]
pub enum Outcome {
    Changed(Success),
    DryRun,
    Unchanged,
    Failed,
}
fn print_outcome(task_name: String, outcome: Outcome) {

    fn message(task_name: &str, outcome: &Outcome, interactive: bool) -> String {
        let tick = if interactive { " ✅ " } else { "Task succeeded: " };
        let cross = if interactive { " ❌ " } else { "Task failed: " };
        let unimportant = console::Style::new().color256(243); //gray

        match outcome {
            Outcome::Changed(success)   => {
                let mut message = format!("{tick}{task_name}");
                if let Some(success_message) = &success.message {
                    message.push_str(&format!(" ({success_message})"));
                }
                message
            }
            Outcome::DryRun => {
                format!("{tick}{task_name} (Needs Change)")
            }
            Outcome::Unchanged => {
                let mut message = format!("{task_name} (Unchanged)");
                if interactive {
                    message = unimportant.apply_to(message).to_string();
                }
                format!("{tick}{message}")
            }
            Outcome::Failed => {
                format!("{cross}{task_name}")
            }
        }
    }

    println!("{}", message(&task_name, &outcome, console::user_attended()));
    info!("{}", message(&task_name, &outcome, false));
}

