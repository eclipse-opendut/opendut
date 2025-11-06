use std::collections::HashMap;
use std::error::Error;
use std::time::Duration;
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use opendut_viper_rt::common::{Identifier, TestCaseIdentifier, TestIdentifier};
use opendut_viper_rt::run::{RunEvent, RunState, TestSuiteRunState};
use crate::console::RenderState;

pub fn render_run_event(
    event: RunEvent,
    render_state: &mut RenderState,
) -> Result<(), Box<dyn Error>> {

    let RenderState {
        progress_bars,
        current_test_suite_state,
        passed_tests_amount,
        failed_tests,
        live_progress_bar,
        ..
    } = render_state;

    match event {
        RunEvent::Initialized(state) => {

            if let Some(progress_bar) = progress_bars.get(state.identifier.as_str()) {
                progress_bar.tick()
            }

            current_test_suite_state.replace(state);
        }
        RunEvent::TestSuiteStarted(_) => {
            update_run_state(TestKind::TestSuite, current_test_suite_state, RunState::Running);
        }
        RunEvent::TestSuitePassed(_) => {
            update_run_state(TestKind::TestSuite, current_test_suite_state, RunState::Passed);
        }
        RunEvent::TestSuiteFailed(_) => {
            update_run_state(TestKind::TestSuite, current_test_suite_state, RunState::Failed);
        }
        RunEvent::TestCaseStarted(identifier) => {
            update_run_state(TestKind::TestCase(identifier), current_test_suite_state, RunState::Running);
        }
        RunEvent::TestCasePassed(identifier) => {
            update_run_state(TestKind::TestCase(identifier), current_test_suite_state, RunState::Passed);
        }
        RunEvent::TestCaseFailed(identifier) => {
            update_run_state(TestKind::TestCase(identifier), current_test_suite_state, RunState::Failed);
        }
        RunEvent::TestStarted(identifier) => {
            update_run_state(TestKind::Test(identifier), current_test_suite_state, RunState::Running);
        }
        RunEvent::TestPassed(identifier) => {
            *passed_tests_amount += 1;
            live_progress_bar.inc(1);
            update_run_state(TestKind::Test(identifier), current_test_suite_state, RunState::Passed);
        }
        RunEvent::TestFailed(identifier) => {
            failed_tests.push(identifier.to_string());
            live_progress_bar.inc(1);
            update_run_state(TestKind::Test(identifier), current_test_suite_state, RunState::Failed);
        }
    }

    if let Some(test_suite_state) = current_test_suite_state.as_ref() {
        update(test_suite_state, progress_bars);
    }

    Ok(())
}

fn update(
    test_suite_state: &TestSuiteRunState,
    progress_bars: &HashMap<String, ProgressBar>,
) {
    update_progressbar(progress_bars, &test_suite_state.identifier, test_suite_state.state);

    for test_case_state in test_suite_state.cases.iter() {
        update_progressbar(progress_bars, &test_case_state.identifier, test_case_state.state);

        for test_state in test_case_state.tests.iter() {
            update_progressbar(progress_bars, &test_state.identifier, test_state.state);
        }
    }
}

fn update_progressbar<I: Identifier>(
    progress_bars: &HashMap<String, ProgressBar>,
    identifier: &I,
    run_state: RunState
) {
    if let Some(bar) = progress_bars.get(identifier.as_str()) {
        match run_state {
            RunState::Running => {
                let spinner_style = ProgressStyle::with_template("{prefix:.bold}{spinner:.bold.blue} {msg}")
                    .unwrap()
                    .tick_strings(&["⢎ ", "⠎⠁", "⠊⠑", "⠈⠱", " ⡱", "⢀⡰", "⢄⡠", "⢆⡀", "  "]);
                bar.set_style(spinner_style);
                bar.set_message(String::from(identifier.name()));
                bar.enable_steady_tick(Duration::from_millis(100));
            }
            RunState::Passed => {
                bar.set_style(ProgressStyle::with_template("{prefix:.bold} {msg}").unwrap());
                let message = format!("{} {}", style("✔").green().bold(), identifier.name());
                bar.finish_with_message(message);
            }
            RunState::Failed => {
                bar.set_style(ProgressStyle::with_template("{prefix:.bold} {msg}").unwrap());
                let message = format!("{} {}", style("✗").red().bold(), identifier.name());
                bar.finish_with_message(message);
            }
            _ => {}
        }
    }
}

enum TestKind {
    TestSuite,
    TestCase(TestCaseIdentifier),
    Test(TestIdentifier),
}

fn update_run_state(
    test_kind: TestKind,
    current_test_suite_state: &mut Option<TestSuiteRunState>,
    run_state: RunState
) {
    match test_kind {
        TestKind::TestSuite => {
            let test_suite_state = current_test_suite_state.as_mut()
                .expect("There must be a TestSuite!");
            test_suite_state.state = run_state;
        }
        TestKind::TestCase(identifier) => {
            let test_case_state = current_test_suite_state.as_mut()
                .expect("There must be a TestSuite!")
                .cases.iter_mut()
                .find(|case| case.identifier == identifier)
                .expect("There must be a TestCase!");

            test_case_state.state = run_state;
        }
        TestKind::Test(identifier) => {
            let test_state = current_test_suite_state.as_mut()
                .expect("There must be a TestSuite!")
                .cases.iter_mut()
                .flat_map(|case| case.tests.iter_mut())
                .find(|test| test.identifier == identifier)
                .expect("There must be a Test!");

            test_state.state = run_state;
        }
    }
}
