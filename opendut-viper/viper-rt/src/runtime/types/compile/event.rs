use crate::common::{TestCaseIdentifier, TestIdentifier, TestSuiteIdentifier};

#[derive(Clone, Debug, PartialEq)]
pub enum CompileEvent {
    CompilationStarted(TestSuiteIdentifier),
    CompilationPassed(CompilationSummary),
    CompilationFailed(TestSuiteIdentifier),
}

#[derive(Clone, Debug, PartialEq)]
pub struct CompilationSummary {
    pub suite: CompiledTestSuite,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CompiledTestSuite {
    pub identifier: TestSuiteIdentifier,
    pub cases: Vec<CompiledTestCase>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CompiledTestCase {
    pub identifier: TestCaseIdentifier,
    pub tests: Vec<CompiledTest>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CompiledTest {
    pub identifier: TestIdentifier,
}
