use std::io;
use std::io::Write;
use std::time::Duration;

use indicatif::{ProgressBar, ProgressStyle};
use tracing::{error, info};

use opendut_util::project;

use crate::common::task::{Success, Task, TaskFulfilled};

const DRY_RUN_BANNER: &str = r"
                Running in
             Development mode
                    --
          Activating --dry-run to
        prevent changes to the system.
        ";

pub async fn run(run_mode: RunMode, no_confirm: bool, tasks: &[Box<dyn Task>]) -> anyhow::Result<()> {
    let run_mode = if project::is_running_in_development() {
        println!("{DRY_RUN_BANNER}");
        info!("{DRY_RUN_BANNER}");
        RunMode::DryRun
    } else {
        run_mode
    };

    if run_mode != RunMode::DryRun {
        sudo::with_env(&["OPENDUT_EDGAR_"]) //Request before doing anything else, as it restarts the process when sudo is not present.
            .expect("Failed to request sudo privileges.");
    }
    if no_confirm || user_confirmation(run_mode)? {
        run_tasks(tasks, run_mode);
    }
    println!();
    Ok(())
}

fn user_confirmation(run_mode: RunMode) -> anyhow::Result<bool> {
    match run_mode {
        RunMode::DryRun => {
            println!("Pretending to setup EDGAR on your system.");
            Ok(true)
        }
        RunMode::Normal => {
            println!("This will setup EDGAR on your system.");
            print!("Do you want to continue? [Y/n] ");
            io::stdout().flush()?;
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;

            match input.trim().to_lowercase().as_ref() {
                "" | "y" | "yes" => Ok(true),
                _ => {
                    println!("Aborting.");
                    info!("Aborting, because user did not confirm execution.");
                    Ok(false)
                }
            }
        }
    }
}

fn run_tasks(
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

        let is_fulfilled = match task.check_fulfilled() {
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
                if run_mode == RunMode::DryRun {
                    Outcome::DryRun
                } else {
                    let result = task.execute();
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
            match task.check_fulfilled() {
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
pub enum RunMode { Normal, DryRun }

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


#[cfg(test)]
pub mod test {
    use crate::common::task::{Task, TaskFulfilled};

    pub fn unchecked(task: impl Task) -> anyhow::Result<()> {
        assert_eq!(task.check_fulfilled()?, TaskFulfilled::Unchecked);
        task.execute()?;
        assert_eq!(task.check_fulfilled()?, TaskFulfilled::Unchecked);
        Ok(())
    }
}
