use std::error::Error;
use std::time::Duration;
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use crate::console::RenderState;

pub fn create_summary_progress_bar(render_state: &mut RenderState) -> Result<(), Box<dyn Error>> {
    let RenderState { multi_progress, .. } = render_state;

    let summary_bar = multi_progress.add(ProgressBar::new(1))
        .with_style(ProgressStyle::with_template("\n {prefix:.bold.blue} {elapsed:.bold.blue} {msg}").unwrap())
        .with_prefix("⏱");
    summary_bar.enable_steady_tick(Duration::from_millis(100));

    render_state.summary_bar = summary_bar;

    set_summary_message(render_state)
}

pub fn create_live_progress_bar(render_state: &mut RenderState) -> Result<(), Box<dyn Error>> {
    let RenderState { multi_progress, test_amount, .. } = render_state;

    let new_live_progress_bar = multi_progress.add(ProgressBar::new(*test_amount))
        .with_style(ProgressStyle::with_template(
            "\n {bar:40.blue} {pos}/{len} tests finished")?
            .progress_chars("■□")
        );

    render_state.live_progress_bar = new_live_progress_bar;

    Ok(())
}

pub fn set_summary_message(render_state: &RenderState) -> Result<(), Box<dyn Error>> {
    let RenderState { summary_bar, passed_tests_amount, failed_tests,  .. } = render_state;
    const SPACE: &str = "    ";

    let mut message = String::new();

    let executed_tests_count = failed_tests.len() as u32 + passed_tests_amount;
    message.push_str(&format!("| {} {} ", style("▶").cyan().bold(), style(executed_tests_count).cyan().bold()));

    message.push_str(
        &format!(
            "| {} {} ",
            style("✔").green().bold(),
            style(passed_tests_amount).green().bold()
        )
    );

    message.push_str(
        &format!(
            "| {} {} ",
            style("✗").red().bold(),
            style(failed_tests.len()).red().bold()
        )
    );

    if !failed_tests.is_empty() {
        message.push_str(&format!("\n\n {} test(s) failed: ", style(failed_tests.len()).red().bold()));
        for test_name in failed_tests {
            message.push_str(&format!("\n{}- {test_name}", SPACE.repeat(2)));
        }
    }

    message.push('\n');

    summary_bar.set_message(message);

    Ok(())
}
