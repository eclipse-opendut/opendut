use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;
use crate::setup::RunMode;

pub(crate) struct ProgressBarForCLI {
    progress_bar: Option<ProgressBar>,
}

impl ProgressBarForCLI {
    pub(crate) fn new(task_name: String, run_mode: RunMode, style: ProgressStyle) -> Self {
        let spinner = match run_mode {
            RunMode::Service => {
                None
            }
            _ => {
                let spinner = ProgressBar::new_spinner();
                spinner.enable_steady_tick(Duration::from_millis(120));
                spinner.set_style(style.clone());
                spinner.set_message(task_name);
                Some(spinner)
            }
        };
        Self { progress_bar: spinner }
    }

    pub(crate) fn progress_style() -> ProgressStyle {
        ProgressStyle::with_template(" {spinner:.dim}  {msg}").unwrap()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏", ""])
    }

    pub(crate) fn finish_and_clear(&self) {
        if let Some(progress_bar) = self.progress_bar.as_ref() {
            progress_bar.finish_and_clear();
        }
    }
}