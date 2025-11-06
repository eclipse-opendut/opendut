mod summary;
mod render_event;
mod run_event;

use std::collections::HashMap;
use std::error::Error;
use futures::{Stream, StreamExt};
use indicatif::{MultiProgress, ProgressBar};
use opendut_viper_rt::run::{RunEvent, TestSuiteRunState};
use opendut_viper_rt::compile::{CompilationSummary, CompileEvent};

#[derive(Debug)]
#[allow(clippy::enum_variant_names)]
pub enum Event {
    CompileEvent(CompileEvent),
    RenderEvent,
    RunEvent(RunEvent),
}

struct RenderState {
    compiled_test_suites: Vec<CompilationSummary>,
    current_test_suite_state: Option<TestSuiteRunState>,
    failed_tests: Vec<String>,
    live_progress_bar: ProgressBar,
    multi_progress: MultiProgress,
    passed_tests_amount: u32,
    progress_bars: HashMap<String, ProgressBar>,
    summary_bar: ProgressBar,
    test_amount: u64,
}

impl Default for RenderState {
    fn default() -> Self {
        Self {
            compiled_test_suites: Vec::new(),
            current_test_suite_state: None,
            failed_tests: Vec::new(),
            live_progress_bar: ProgressBar::hidden(),
            multi_progress: Default::default(),
            passed_tests_amount: 0,
            progress_bars: Default::default(),
            summary_bar: ProgressBar::hidden(),
            test_amount: 0,
        }
    }
}

pub async fn render(mut events: impl Stream<Item = Event> + Unpin) -> Result<(), Box<dyn Error>> {

    let mut render_state = RenderState::default();

    while let Some(event) = events.next().await {
        match event {
            Event::CompileEvent(CompileEvent::CompilationPassed(summary)) => {
                render_state.compiled_test_suites.push(summary);
            }
            Event::CompileEvent(_) => {}
            Event::RenderEvent => {
                render_event::initial_render(&mut render_state)?
            }
            Event::RunEvent(event) => {
                run_event::render_run_event(event, &mut render_state)?;
                summary::set_summary_message(&render_state)?;
            },
        }
    }

    render_state.live_progress_bar.finish_and_clear();
    render_state.summary_bar.finish();

    Ok(())
}
