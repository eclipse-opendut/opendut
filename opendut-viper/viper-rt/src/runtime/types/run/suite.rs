#[cfg(feature = "py")]
use rustpython_vm as vm;

use crate::runtime::types::naming::{TestCaseIdentifier, TestIdentifier, TestSuiteIdentifier};

pub struct TestSuiteRun {
    pub identifier: TestSuiteIdentifier,
    pub cases: Vec<TestCaseRun>
}

pub struct TestCaseRun {
    pub identifier: TestCaseIdentifier,
    pub tests: Vec<TestRun>,
    #[cfg(feature = "py")] pub ty: vm::PyRef<vm::builtins::PyType>,
    #[cfg(feature = "py")] pub instance: vm::PyObjectRef,
    #[cfg(feature = "py")] pub setup_class_fn: Option<vm::PyObjectRef>,
    #[cfg(feature = "py")] pub teardown_class_fn: Option<vm::PyObjectRef>,

}

pub struct TestRun {
    pub identifier: TestIdentifier,
    #[cfg(feature = "py")] pub test_fn: vm::PyObjectRef,
    #[cfg(feature = "py")] pub setup_fn: Option<vm::PyObjectRef>,
    #[cfg(feature = "py")] pub teardown_fn: Option<vm::PyObjectRef>,
}
