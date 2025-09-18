use crate::common::{TestCaseIdentifier, TestIdentifier, TestSuiteIdentifier};
use crate::run::{RunEvent, TestSuiteRunState};
use crate::runtime::emitter::EventEmitter;
use crate::runtime::types::run::error::{RunError, RunResult};
use futures::TryFutureExt;

pub async fn initialized(
    emitter: &mut dyn EventEmitter<RunEvent>,
    state: TestSuiteRunState
) -> RunResult<()> {
    let suite = Clone::clone(&state.identifier);
    emitter.emit(RunEvent::Initialized(state))
        .map_err(|error| Box::new(RunError::new_failed_event_emission_error(suite, format!("Failed to emit `Initialized` event: {}", error.cause))))
        .await
}

pub async fn test_suite_started(
    emitter: &mut dyn EventEmitter<RunEvent>,
    suite: TestSuiteIdentifier
) -> RunResult<()> {
    emitter.emit(RunEvent::TestSuiteStarted(Clone::clone(&suite)))
        .map_err(|_| Box::new(RunError::new_failed_event_emission_error(suite, "Failed to emit `TestSuiteStarted` event.")))
        .await
}

pub async fn test_suite_passed(
    emitter: &mut dyn EventEmitter<RunEvent>,
    suite: TestSuiteIdentifier
) -> RunResult<()> {
    emitter.emit(RunEvent::TestSuitePassed(Clone::clone(&suite)))
        .map_err(|_| Box::new(RunError::new_failed_event_emission_error(suite, "Failed to emit `TestSuitePassed` event.")))
        .await
}

pub async fn test_suite_failed(
    emitter: &mut dyn EventEmitter<RunEvent>,
    suite: TestSuiteIdentifier
) -> RunResult<()> {
    emitter.emit(RunEvent::TestSuiteFailed(Clone::clone(&suite)))
        .map_err(|_| Box::new(RunError::new_failed_event_emission_error(suite, "Failed to emit `TestSuiteFailed` event.")))
        .await
}

pub async fn test_case_started(
    emitter: &mut dyn EventEmitter<RunEvent>,
    case: TestCaseIdentifier
) -> RunResult<()> {
    emitter.emit(RunEvent::TestCaseStarted(Clone::clone(&case)))
        .map_err(|_| Box::new(RunError::new_failed_event_emission_error(case, "Failed to emit `TestCaseStarted` event.")))
        .await
}

pub async fn test_case_passed(
    emitter: &mut dyn EventEmitter<RunEvent>,
    case: TestCaseIdentifier
) -> RunResult<()> {
    emitter.emit(RunEvent::TestCasePassed(Clone::clone(&case)))
        .map_err(|_| Box::new(RunError::new_failed_event_emission_error(case, "Failed to emit `TestCasePassed` event.")))
        .await
}

pub async fn test_case_failed(
    emitter: &mut dyn EventEmitter<RunEvent>,
    case: TestCaseIdentifier
) -> RunResult<()> {
    emitter.emit(RunEvent::TestCaseFailed(Clone::clone(&case)))
        .map_err(|_| Box::new(RunError::new_failed_event_emission_error(case, "Failed to emit `TestCaseFailed` event.")))
        .await
}

pub async fn test_started(
    emitter: &mut dyn EventEmitter<RunEvent>,
    test: TestIdentifier
) -> RunResult<()> {
    emitter.emit(RunEvent::TestStarted(Clone::clone(&test)))
        .map_err(|_| Box::new(RunError::new_failed_event_emission_error(test, "Failed to emit `TestStarted` event.")))
        .await
}

pub async fn test_passed(
    emitter: &mut dyn EventEmitter<RunEvent>,
    test: TestIdentifier
) -> RunResult<()> {
    emitter.emit(RunEvent::TestPassed(Clone::clone(&test)))
        .map_err(|_| Box::new(RunError::new_failed_event_emission_error(test, "Failed to emit `TestPassed` event.")))
        .await
}

pub async fn test_failed(
    emitter: &mut dyn EventEmitter<RunEvent>,
    test: TestIdentifier
) -> RunResult<()> {
    emitter.emit(RunEvent::TestFailed(Clone::clone(&test)))
        .map_err(|_| Box::new(RunError::new_failed_event_emission_error(test, "Failed to emit `TestFailed` event.")))
        .await
}
