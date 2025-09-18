use crate::common::Identifier;
use crate::run::{Complete, ParameterBindings};
use crate::runtime::ctx::Context;
use crate::runtime::run::py;
use crate::runtime::run::RunError;
use crate::runtime::types::compile::suite::TestCase;
use crate::runtime::types::naming::TestSuiteIdentifier;
use crate::runtime::types::py::error::PythonReflectionError;
use crate::runtime::types::run::error::RunResult;
use crate::runtime::types::run::suite::{TestCaseRun, TestRun, TestSuiteRun};
use rustpython_vm::builtins::{PyModule, PyStr, PyType, PyTypeRef};
use rustpython_vm::{PyObjectRef, PyRef, VirtualMachine};
use std::rc::Rc;
use tracing::debug;

pub fn instantiate(
    suite: &TestSuiteIdentifier,
    module: PyRef<PyModule>,
    context: &Context,
    bindings: Rc<ParameterBindings<Complete>>,
    test_cases: &[TestCase],
    vm: &VirtualMachine,
) -> RunResult<TestSuiteRun> {

    debug!("Preparing test suite '{}'.", suite);

    let mut test_cases_runs = Vec::with_capacity(test_cases.len());

    for (key, value) in module.dict().into_iter() {

        let name = key.downcast_ref::<PyStr>()
            .ok_or_else(|| PythonReflectionError::new_downcast_error(&key, "PyStr"))
            .map_err(|cause| RunError::new_python_reflection_error(Clone::clone(suite), cause))?;

        let Some(case) = test_cases.iter()
            .find(|case| case.identifier.name() == name.as_str())
        else {
            continue;
        };

        let TestCase { identifier, description: _description, setup_fn, teardown_fn, setup_class_fn, teardown_class_fn, tests } = case;

        let ty = value.downcast_ref::<PyType>()
            .ok_or_else(|| PythonReflectionError::new_downcast_error(&value, "PyType"))
            .map_err(|cause| RunError::new_python_reflection_error(Clone::clone(identifier), cause))?
            .to_owned();

        let bindings = Rc::clone(&bindings);

        let instance = make_test_case_instance(&ty, context, bindings, vm)
            .map_err(|cause| RunError::new_python_reflection_error(Clone::clone(identifier), cause))?;

        let tests = tests.iter()
            .map(|test| {
                TestRun {
                    identifier: Clone::clone(&test.identifier),
                    test_fn: Clone::clone(&test.function),
                    setup_fn: Clone::clone(setup_fn),
                    teardown_fn: Clone::clone(teardown_fn),
                }
            })
            .collect::<Vec<_>>();

        test_cases_runs.push(TestCaseRun {
            identifier: Clone::clone(identifier),
            ty,
            instance,
            setup_class_fn: Clone::clone(setup_class_fn),
            teardown_class_fn: Clone::clone(teardown_class_fn),
            tests,
        });
    }

    debug!("Prepared test suite '{}'.", suite);

    Ok(TestSuiteRun { identifier: suite.to_owned(), cases: test_cases_runs })
}

fn make_test_case_instance(
    class: &PyTypeRef,
    context: &Context,
    bindings: Rc<ParameterBindings<Complete>>,
    vm: &VirtualMachine,
) -> Result<PyObjectRef, PythonReflectionError> {
    let instance = vm.ctx.new_base_object(Clone::clone(class), Some(vm.ctx.new_dict()));
    instance.set_attr("parameters", py::make_parameters_dict(bindings, vm), vm)
        .map_err(|_| {
            let owner = instance.str(vm).expect("Invoke `__str__` on object");
            PythonReflectionError::new_attribute_not_writable_error(owner.to_string(), "parameters")
        })?;
    #[cfg(feature = "containers")]
    instance.set_attr("container", py::make_container_runtime_proxy_object(context.container_runtime.as_ref(), vm), vm)
        .map_err(|_| {
            let owner = instance.str(vm).expect("Invoke `__str__` on object");
            PythonReflectionError::new_attribute_not_writable_error(owner.to_string(), "container")
        })?;
    Ok(instance)
}
