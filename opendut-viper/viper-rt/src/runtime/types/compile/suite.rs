use crate::compile::ApiVersion;
use crate::runtime::types::naming::{TestCaseIdentifier, TestIdentifier, TestSuiteIdentifier};

use crate::common::Identifier;
use std::fmt::{Debug, Formatter};

#[cfg(feature = "py")]
use rustpython_vm as vm;

/// A `TestSuite` describes an environment for a collection of [`TestCases`][`TestCase`].
///
/// The `TestSuite` is the top level entity in the hierarchy and contains \[0..n\] [`TestCases`][`TestCase`].
///
/// ```text
/// ┌───────────────────────────────────┐
/// │ Test Suite                        │
/// │                                   │
/// │     ┌──────────────────────────┐  │
/// │     │ Test Case                │  │
/// │     │                          │  │
/// │     │     ┌─────────────────┐  │  │
/// │     │     │ Test            │  │  │
/// │     │     └─────────────────┘  │  │
/// │     └──────────────────────────┘  │
/// └───────────────────────────────────┘
/// ```
/// A `TestSuite` is part of a [`Compilation`] and can be retrieved from its getter-functions or by
/// splitting a [`Compilation`] into its parts.
///
/// [`Compilation`]: crate::compile::Compilation
///
#[cfg_attr(not(feature = "py"), derive(Clone))]
pub struct TestSuite {
    pub(crate) identifier: TestSuiteIdentifier,
    pub(crate) version: ApiVersion,
    pub(crate) cases: Vec<TestCase>,
    #[cfg(feature = "py")] pub(crate) interpreter: vm::Interpreter,
    #[cfg(feature = "py")] pub(crate) module: vm::PyRef<vm::builtins::PyModule>,
}

impl TestSuite {

    pub fn identifier(&self) -> &TestSuiteIdentifier {
        &self.identifier
    }

    pub fn name(&self) -> &str {
        self.identifier.name()
    }

    pub fn test_cases(&self) -> &[TestCase] {
        self.cases.as_slice()
    }
}

impl Debug for TestSuite {

    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        fmt.debug_struct("TestSuite")
            .field("identifier", &self.identifier.to_string())
            .field("api_version", &self.version.to_string())
            .field("cases", &self.cases)
            .finish()
    }
}

/// A `TestCase` groups [`Tests`][`Test`] logically.
///
/// A `TestCase` is a part of a [`TestSuite`] and is the mid-level entity in the hierarchy. It contains \[0..n\] [`Tests`][`Test`].
///
/// ```text
/// ┌───────────────────────────────────┐
/// │ Test Suite                        │
/// │                                   │
/// │     ┌──────────────────────────┐  │
/// │     │ Test Case                │  │
/// │     │                          │  │
/// │     │     ┌─────────────────┐  │  │
/// │     │     │ Test            │  │  │
/// │     │     └─────────────────┘  │  │
/// │     └──────────────────────────┘  │
/// └───────────────────────────────────┘
/// ```
///
#[cfg_attr(not(feature = "py"), derive(Clone))]
pub struct TestCase {
    pub(crate) identifier: TestCaseIdentifier,
    pub(crate) description: Option<String>,
    pub(crate) tests: Vec<Test>,
    #[cfg(feature = "py")] pub(crate) setup_fn: Option<vm::PyObjectRef>,
    #[cfg(feature = "py")] pub(crate) teardown_fn: Option<vm::PyObjectRef>,
    #[cfg(feature = "py")] pub(crate) setup_class_fn: Option<vm::PyObjectRef>,
    #[cfg(feature = "py")] pub(crate) teardown_class_fn: Option<vm::PyObjectRef>,
}

impl TestCase {

    pub fn identifier(&self) -> &TestCaseIdentifier {
        &self.identifier
    }

    pub fn name(&self) -> &str {
        self.identifier.name()
    }

    pub fn description(&self) -> Option<&String> {
        self.description.as_ref()
    }

    pub fn tests(&self) -> &[Test] {
        self.tests.as_slice()
    }
}

impl Debug for TestCase {

    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TestCase")
            .field("name", &self.identifier.to_string())
            .field("tests", &self.tests)
            .finish()
    }
}

/// A `Test` denotes a test function.
///
/// A `Test` is a part of a [`TestCase`] and is at the end of the hierarchy.
///
/// ```text
/// ┌───────────────────────────────────┐
/// │ Test Suite                        │
/// │                                   │
/// │     ┌──────────────────────────┐  │
/// │     │ Test Case                │  │
/// │     │                          │  │
/// │     │     ┌─────────────────┐  │  │
/// │     │     │ Test            │  │  │
/// │     │     └─────────────────┘  │  │
/// │     └──────────────────────────┘  │
/// └───────────────────────────────────┘
/// ```
///
#[cfg_attr(not(feature = "py"), derive(Clone))]
pub struct Test {
    pub(crate) identifier: TestIdentifier,
    #[cfg(feature = "py")] pub(crate) function: vm::PyObjectRef
}

impl Test {

    pub fn identifier(&self) -> &TestIdentifier {
        &self.identifier
    }

    pub fn name(&self) -> &str {
        self.identifier.name()
    }
}

impl Debug for Test {

    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Test")
            .field("name", &self.identifier.to_string())
            .finish()
    }
}
