use crate::common::{TestCaseIdentifier, TestIdentifier, TestSuiteIdentifier};
use crate::runtime::types::run::suite::{TestCaseRun, TestRun, TestSuiteRun};

#[derive(Clone, Debug, PartialEq)]
pub enum RunEvent {
    Initialized(TestSuiteRunState),
    TestSuiteStarted(TestSuiteIdentifier),
    TestSuitePassed(TestSuiteIdentifier),
    TestSuiteFailed(TestSuiteIdentifier),
    TestCaseStarted(TestCaseIdentifier),
    TestCasePassed(TestCaseIdentifier),
    TestCaseFailed(TestCaseIdentifier),
    TestStarted(TestIdentifier),
    TestPassed(TestIdentifier),
    TestFailed(TestIdentifier),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RunState {
    Initialized,
    Ignored,
    Running,
    Passed,
    Failed,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TestSuiteRunState {
    pub identifier: TestSuiteIdentifier,
    pub state: RunState,
    pub cases: Vec<TestCaseRunState>
}

impl TestSuiteRunState {

    pub fn from_run(suite: &TestSuiteRun) -> Self {
        TestSuiteRunState {
            identifier: Clone::clone(&suite.identifier),
            state: RunState::Initialized,
            cases: suite.cases.iter().map(TestCaseRunState::from_run).collect()
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TestCaseRunState {
    pub identifier: TestCaseIdentifier,
    pub state: RunState,
    pub tests: Vec<TestRunState>
}

impl TestCaseRunState {

    pub fn from_run(case: &TestCaseRun) -> Self {
        TestCaseRunState {
            identifier: Clone::clone(&case.identifier),
            state: RunState::Initialized,
            tests: case.tests.iter().map(TestRunState::from_run).collect()
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TestRunState {
    pub identifier: TestIdentifier,
    pub state: RunState,
}

impl TestRunState {

    pub fn from_run(test: &TestRun) -> Self {
        TestRunState {
            identifier: Clone::clone(&test.identifier),
            state: RunState::Initialized,
        }
    }
}
