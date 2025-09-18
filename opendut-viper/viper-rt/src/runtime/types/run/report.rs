use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use crate::runtime::types::naming::{TestCaseIdentifier, TestIdentifier, TestSuiteIdentifier};

#[derive(Clone, Debug)]
pub struct TestSuiteReport {
    pub name: TestSuiteIdentifier,
    pub cases: Vec<TestCaseReport>
}

#[derive(Clone, Debug)]
pub struct TestCaseReport {
    pub name: TestCaseIdentifier,
    pub tests: Vec<TestReport>
}

#[derive(Clone, Debug)]
pub struct TestReport {
    pub identifier: TestIdentifier,
    pub outcome: Outcome,
    pub properties: Vec<ReportProperty>,
    pub output: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Outcome {
    Success,
    Failure,
}

impl Outcome {
    pub fn is_failure(&self) -> bool {
        matches!(self, Outcome::Failure)
    }
}

impl Display for Outcome {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Outcome::Success => write!(f, "Success"),
            Outcome::Failure => write!(f, "Failure"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ReportProperty {
    pub name: String,
    pub value: ReportPropertyValue
}

#[derive(Clone, Debug, PartialEq)]
pub enum ReportPropertyValue {
    Number(i64),
    String(String),
    File(PathBuf),
}

pub trait Report {

    fn outcome(&self) -> Outcome;

    fn is_success(&self) -> bool {
        self.outcome() == Outcome::Success
    }

    fn is_failure(&self) -> bool {
        self.outcome() == Outcome::Failure
    }
}

impl Report for TestSuiteReport {

    fn outcome(&self) -> Outcome {
        if self.cases.iter().any(|report| report.outcome().is_failure()) {
            Outcome::Failure
        }
        else {
            Outcome::Success
        }
    }
}

impl Report for TestCaseReport {

    fn outcome(&self) -> Outcome {
        if self.tests.iter().any(|run| run.outcome().is_failure()) {
            Outcome::Failure
        }
        else {
            Outcome::Success
        }
    }
}

impl Report for TestReport {
    fn outcome(&self) -> Outcome {
        self.outcome
    }
}
