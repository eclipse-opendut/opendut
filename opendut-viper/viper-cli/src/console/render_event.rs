use std::error::Error;
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use opendut_viper_rt::common::Identifier;
use opendut_viper_rt::compile::CompilationSummary;
use crate::console::{summary, RenderState};

pub fn initial_render(render_state: &mut RenderState) -> Result<(), Box<dyn Error>> {
    let summaries = std::mem::take(&mut render_state.compiled_test_suites);
    
    render_state.test_amount = summaries
        .iter()
        .flat_map(|summary| &summary.suite.cases)
        .map(|cases| cases.tests.len())
        .sum::<usize>() as u64;

    render_state.live_progress_bar.set_length(render_state.test_amount);

    for summary in summaries {
        create_progress_tree(&summary, render_state);
    }

    summary::create_live_progress_bar(render_state)?;
    summary::create_summary_progress_bar(render_state)?;
    
    Ok(())
}

fn create_progress_tree(summary: &CompilationSummary, render_state: &mut RenderState) {
    const SPACER: &str = "   ";
    create_progressbar(render_state, &summary.suite.identifier, None, None);

    for (case_index, case) in summary.suite.cases.iter().enumerate() {
        let is_last_case = case_index == summary.suite.cases.len() - 1;
        create_progressbar(render_state, &case.identifier, Some(SPACER.to_string()), Some(is_last_case));

        for (test_index, test) in case.tests.iter().enumerate() {
            let case_prefix = if is_last_case { SPACER.repeat(2) } else { format!("│  {SPACER}")};
            let test_prefix = format!("{SPACER}{case_prefix}");
            let is_last_test = test_index == case.tests.len() - 1;
            create_progressbar(render_state, &test.identifier, Some(test_prefix), Some(is_last_test));
        }
    }
}

fn create_progressbar<I: Identifier>(
    render_state: &mut RenderState,
    identifier: &I,
    prefix: Option<String>,
    is_last: Option<bool>,
) {

    let RenderState { multi_progress, progress_bars, .. } = render_state;

    let branch = match is_last {
        Some(true) => "└─ ",
        Some(false) => "├─ ",
        None => "",
    };
    let bar_prefix = format!("{}{}", prefix.unwrap_or_default(), branch);

    let bar = multi_progress.add(ProgressBar::new_spinner())
        .with_style(ProgressStyle::with_template("{prefix:.bold}   {wide_msg}").unwrap())
        .with_prefix(bar_prefix)
        .with_message(format!("{}: {}", identifier.name(), style("Compiled").dim().bold()));

    bar.tick();

    progress_bars.insert(identifier.to_string(), bar);
}
