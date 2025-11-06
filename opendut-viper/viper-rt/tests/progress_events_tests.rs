use futures::StreamExt;
use googletest::prelude::*;
use indoc::indoc;
use tokio_stream::wrappers::ReceiverStream;
use tokio_util::sync::PollSender;
use opendut_viper_rt::common::{TestCaseIdentifier, TestIdentifier};
use opendut_viper_rt::compile::{Compilation, CompilationError, CompilationErrorKind, CompilationSummary, CompileEvent, CompileResult, CompiledTest, CompiledTestCase, CompiledTestSuite, IdentifierFilter};
use opendut_viper_rt::events::{emitter, EventEmitter};
use opendut_viper_rt::run::{ParameterBindings, RunError, RunErrorKind, RunEvent, RunState, TestCaseRunState, TestRunState, TestSuiteRunState};
use opendut_viper_rt::source::Source;
use opendut_viper_rt::ViperRuntime;

async fn compile_test(runtime: &ViperRuntime, source: &Source, emitter: &mut dyn EventEmitter<CompileEvent>) -> CompileResult<Compilation> {
    runtime.compile(&source, emitter, &IdentifierFilter::default()).await
}

const EXAMPLE_CODE: &str = indoc!(r#"
    # VIPER_VERSION = 1.0
    from viper import unittest

    class MyTestCase(unittest.TestCase):
        def test_awesomeness(self):
            print("Awesome!")
        def test_everything_else(self):
            print("The other stuff!")
    class MyFailingTestCase(unittest.TestCase):
        def test_failure(self):
            self.fail("BOOM")
"#
);

#[tokio::test]
async fn test_that_compile_events_are_emitted() -> Result<()> {

    let runtime = ViperRuntime::default();

    let source = Source::embedded(EXAMPLE_CODE);

    let (tx, rx) = tokio::sync::mpsc::channel::<CompileEvent>(64);

    let _ = compile_test(&runtime, &source, &mut emitter::sink(PollSender::new(tx))).await?.into_suite();

    let events = ReceiverStream::new(rx)
        .collect::<Vec<_>>().await;

    let suite_identifier = source.identifier;
    let case_1_identifier = TestCaseIdentifier::try_from_suite(&suite_identifier, "MyTestCase")?;
    let case_2_identifier = TestCaseIdentifier::try_from_suite(&suite_identifier, "MyFailingTestCase")?;
    let test_1_identifier = TestIdentifier::try_from_case(&case_1_identifier, "test_awesomeness")?;
    let test_2_identifier = TestIdentifier::try_from_case(&case_1_identifier, "test_everything_else")?;
    let test_3_identifier = TestIdentifier::try_from_case(&case_2_identifier, "test_failure")?;

    assert_that!(events, container_eq([
        CompileEvent::CompilationStarted(Clone::clone(&suite_identifier)),
        CompileEvent::CompilationPassed(CompilationSummary {
            suite: CompiledTestSuite {
                identifier: suite_identifier,
                cases: vec! [
                    CompiledTestCase {
                        identifier: Clone::clone(&case_1_identifier),
                        tests: vec![
                            CompiledTest {
                                identifier: Clone::clone(&test_1_identifier),
                            },
                            CompiledTest {
                                identifier: Clone::clone(&test_2_identifier),
                            }
                        ]
                    },
                    CompiledTestCase {
                        identifier: Clone::clone(&case_2_identifier),
                        tests: vec![
                            CompiledTest {
                                identifier: Clone::clone(&test_3_identifier),
                            }
                        ]
                    }
                ]
            }
        }),
    ]));

    Ok(())
}

#[tokio::test]
async fn test_that_run_events_are_emitted() -> Result<()> {

    let runtime = ViperRuntime::default();

    let source = Source::embedded(EXAMPLE_CODE);

    let (tx, rx) = tokio::sync::mpsc::channel::<RunEvent>(64);

    let suite = compile_test(&runtime, &source, &mut emitter::drain()).await?.into_suite();

    let suite_identifier = Clone::clone(suite.identifier());
    let case_1_identifier = Clone::clone(suite.test_cases()[0].identifier());
    let case_2_identifier = Clone::clone(suite.test_cases()[1].identifier());
    let test_1_identifier = Clone::clone(suite.test_cases()[0].tests()[0].identifier());
    let test_2_identifier = Clone::clone(suite.test_cases()[0].tests()[1].identifier());
    let test_3_identifier = Clone::clone(suite.test_cases()[1].tests()[0].identifier());

    let _ = runtime.run(suite, ParameterBindings::new(), &mut emitter::sink(PollSender::new(tx))).await?;

    let events = ReceiverStream::new(rx)
        .collect::<Vec<_>>().await;

    assert_that!(events, container_eq([
        RunEvent::Initialized(TestSuiteRunState {
            identifier: Clone::clone(&suite_identifier),
            state: RunState::Initialized,
            cases: vec![
                TestCaseRunState {
                    identifier: Clone::clone(&case_1_identifier),
                    state: RunState::Initialized,
                    tests: vec![
                        TestRunState {
                            identifier: Clone::clone(&test_1_identifier),
                            state: RunState::Initialized
                        },
                        TestRunState {
                            identifier: Clone::clone(&test_2_identifier),
                            state: RunState::Initialized
                        }
                    ]
                },
                TestCaseRunState {
                    identifier: Clone::clone(&case_2_identifier),
                    state: RunState::Initialized,
                    tests: vec![
                        TestRunState {
                            identifier: Clone::clone(&test_3_identifier),
                            state: RunState::Initialized
                        }
                    ]
                }
            ]
        }),
        RunEvent::TestSuiteStarted(Clone::clone(&suite_identifier)),
        RunEvent::TestCaseStarted(Clone::clone(&case_1_identifier)),
        RunEvent::TestStarted(Clone::clone(&test_1_identifier)),
        RunEvent::TestPassed(test_1_identifier),
        RunEvent::TestStarted(Clone::clone(&test_2_identifier)),
        RunEvent::TestPassed(test_2_identifier),
        RunEvent::TestCasePassed(case_1_identifier),
        RunEvent::TestCaseStarted(Clone::clone(&case_2_identifier)),
        RunEvent::TestStarted(Clone::clone(&test_3_identifier)),
        RunEvent::TestFailed(test_3_identifier),
        RunEvent::TestCaseFailed(Clone::clone(&case_2_identifier)),
        RunEvent::TestSuiteFailed(suite_identifier),
    ]));

    Ok(())
}

#[tokio::test]
async fn test_that_compile_fails_when_event_emission_failed() -> Result<()> {

    let runtime = ViperRuntime::default();

    let source = Source::embedded(EXAMPLE_CODE);

    let result = compile_test(&runtime, &source, &mut emitter::fail()).await;

    assert_that!(result.map_err(|e| *e), err(matches_pattern!(
        CompilationError {
            kind: matches_pattern!(CompilationErrorKind::FailedEventEmission { .. }),
            ..
        }
    )));

    Ok(())
}

#[tokio::test]
async fn test_that_run_fails_when_event_emission_failed() -> Result<()> {

    let runtime = ViperRuntime::default();

    let source = Source::embedded(EXAMPLE_CODE);

    let suite = compile_test(&runtime, &source, &mut emitter::drain()).await?.into_suite();

    let result = runtime.run(suite, ParameterBindings::new(), &mut emitter::fail()).await;

    assert_that!(result.map_err(|e| *e), err(matches_pattern!(
        RunError {
            kind: matches_pattern!(RunErrorKind::FailedEventEmission { .. }),
            ..
        }
    )));

    Ok(())
}
