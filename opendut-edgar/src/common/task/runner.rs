use std::time::Duration;

use indicatif::{ProgressBar, ProgressStyle};
use tracing::{debug, error, info};

use crate::common::task::{Success, Task, TaskFulfilled};

pub async fn run(run_mode: RunMode, tasks: &[Box<dyn Task>]) -> anyhow::Result<()> {
    if tasks.is_empty() {
        debug!("No tasks to run. Skipping.");
        return Ok(())
    }

    let task_names_string = tasks.iter().map(|task| task.description()).collect::<Vec<_>>().join(", ");
    debug!("Running tasks: {task_names_string}");

    run_tasks(tasks, run_mode).await;

    println!();
    debug!("Completed running tasks: {task_names_string}");
    Ok(())
}

async fn run_tasks(
    tasks: &[Box<dyn Task>],
    run_mode: RunMode,
) {
    println!();

    let progress_style = ProgressStyle::with_template(" {spinner:.dim}  {msg}").unwrap()
        .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏", ""]);
    for task in tasks {
        let spinner = ProgressBar::new_spinner();
        spinner.enable_steady_tick(Duration::from_millis(120));
        spinner.set_style(progress_style.clone());
        spinner.set_message(task.description());

        let is_fulfilled = match task.check_fulfilled().await {
            Ok(is_fulfilled) => is_fulfilled,
            Err(cause) => {
                print_outcome(task.description(), Outcome::Failed);
                print_error("Error while determining system state:", Some(cause));
                return;
            }
        };

        let outcome = match is_fulfilled {
            TaskFulfilled::Yes => Outcome::Unchanged,
            TaskFulfilled::No | TaskFulfilled::Unchecked => {
                if run_mode == RunMode::SetupDryRun {
                    Outcome::DryRun
                } else {
                    let result = task.execute().await;
                    spinner.finish_and_clear();
                    match result {
                        Ok(success) => Outcome::Changed(success),
                        Err(cause) => {
                            print_outcome(task.description(), Outcome::Failed);
                            print_error("Error while executing:", Some(cause));
                            return;
                        }
                    }
                }
            }
        };
        spinner.finish_and_clear();

        if let Outcome::Changed(_) = outcome {
            match task.check_fulfilled().await {
                Ok(fulfillment) => match fulfillment {
                    TaskFulfilled::Yes | TaskFulfilled::Unchecked => {}, //do nothing
                    TaskFulfilled::No => {
                        print_outcome(task.description(), Outcome::Failed);
                        print_error("Execution succeeded, but system state check indicated task still needing execution.", None);
                        return;
                    }
                }
                Err(cause) => {
                    print_outcome(task.description(), Outcome::Failed);
                    print_error("Error while determining system state after execution:", Some(cause));
                    return;
                }
            }
        };

        print_outcome(task.description(), outcome)
    }
}

fn print_error(context: impl AsRef<str>, error: Option<anyhow::Error>) {
    let message = {
        let mut message = String::new();
        message.push_str(context.as_ref());
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
}

#[derive(Clone, Copy, PartialEq)]
pub enum RunMode { Setup, SetupDryRun, Service }

enum Outcome {
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
