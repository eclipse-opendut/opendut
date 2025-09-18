use crate::compile::Compilation;
use crate::runtime::emitter::EventEmitter;
use crate::runtime::types::compile::error::CompilationError;
use crate::runtime::types::compile::event::{CompilationSummary, CompileEvent, CompiledTest, CompiledTestCase, CompiledTestSuite};
use crate::source::Source;
use futures::TryFutureExt;

pub async fn compilation_started(
    emitter: &mut dyn EventEmitter<CompileEvent>,
    source: &Source,
) -> Result<(), CompilationError> {
    emitter.emit(CompileEvent::CompilationStarted(Clone::clone(&source.identifier)))
        .map_err(|_| CompilationError::new_failed_event_emission_error(source, "Failed to emit `CompilationStarted` event."))
        .await
}

pub async fn compilation_passed(
    emitter: &mut dyn EventEmitter<CompileEvent>,
    source: &Source,
    compilation: &Compilation
) -> Result<(), CompilationError> {

    let mut compiled_suite = CompiledTestSuite {
        identifier: Clone::clone(compilation.identifier()),
        cases: Vec::with_capacity(compilation.suite().test_cases().len()),
    };
    for case in compilation.suite().test_cases() {
        let mut compiled_case = CompiledTestCase {
            identifier: Clone::clone(&case.identifier),
            tests: Vec::with_capacity(case.tests.len()),
        };
        for test in case.tests() {
            let compiled_test = CompiledTest {
                identifier: Clone::clone(test.identifier()),
            };
            compiled_case.tests.push(compiled_test);
        }
        compiled_suite.cases.push(compiled_case);
    }
    let summary = CompilationSummary {
        suite: compiled_suite,
    };
    emitter.emit(CompileEvent::CompilationPassed(summary))
        .map_err(|_| CompilationError::new_failed_event_emission_error(source, "Failed to emit `CompilationPassed` event."))
        .await
}

pub async fn compilation_failed(
    emitter: &mut dyn EventEmitter<CompileEvent>,
    source: &Source,
) -> Result<(), CompilationError> {
    emitter.emit(CompileEvent::CompilationFailed(Clone::clone(&source.identifier)))
        .map_err(|_| CompilationError::new_failed_event_emission_error(source, "Failed to emit `CompilationFailed` event."))
        .await
}
