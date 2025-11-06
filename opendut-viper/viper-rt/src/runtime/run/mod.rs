mod py;
mod instantiate;
mod emit;

use crate::compile::TestSuite;
use crate::run::{Complete, Outcome, Report, RunEvent, TestReport, TestSuiteReport, TestSuiteRunState};
use crate::runtime::ctx::Context;
use crate::runtime::emitter::EventEmitter;
use crate::runtime::run::instantiate::instantiate;
use crate::runtime::run::py::{make_report_properties, make_stdout_object};
use crate::runtime::types::py::error::PythonReflectionError;
use crate::runtime::types::run::error::RunError;
use crate::runtime::types::run::error::RunResult;
use crate::runtime::types::run::parameters::ParameterBindings;
use crate::runtime::types::run::report::TestCaseReport;
use crate::runtime::types::run::suite::{TestCaseRun, TestRun, TestSuiteRun};
use rustpython_vm::builtins::PyBaseExceptionRef;
use rustpython_vm::function::OptionalArg;
use rustpython_vm::{Interpreter, PyObjectRef, VirtualMachine};
use std::cell::RefCell;
use std::rc::Rc;
use tracing::{debug, error, info};
use opendut_viper_py::file::file::FileHandler;

pub async fn run(
    suite: TestSuite,
    bindings: ParameterBindings<Complete>,
    context: &Context,
    emitter: &mut dyn EventEmitter<RunEvent>
) -> RunResult<TestSuiteReport> {

    let TestSuite { identifier, interpreter, module, cases, .. } = suite;
    let bindings = Rc::new(bindings);

    let test_suite_run = interpreter.enter(|vm| {
        instantiate(&identifier, module, context, bindings, &cases, vm)
    })?;

    let state = TestSuiteRunState::from_run(&test_suite_run);

    emit::initialized(emitter, Clone::clone(&state)).await?;

    run_test_suite(test_suite_run, emitter, &interpreter).await
}

async fn run_test_suite(
    suite: TestSuiteRun,
    emitter: &mut dyn EventEmitter<RunEvent>,
    interpreter: &Interpreter,
) -> RunResult<TestSuiteReport> {

    let TestSuiteRun { identifier, cases } = suite;

    let mut test_suite_report = TestSuiteReport {
        name: Clone::clone(&identifier),
        cases: Vec::with_capacity(cases.len()),
    };

    debug!("Running suite: {}", identifier);

    emit::test_suite_started(emitter, Clone::clone(&identifier)).await?;

    for case in cases {
        let test_case_report = run_test_case(case, emitter, interpreter).await?;
        test_suite_report.cases.push(test_case_report);
    }

    if test_suite_report.is_success() {
        info!("Finished suite '{}' successfully.", identifier);
        emit::test_suite_passed(emitter, identifier).await?;
    }
    else {
        info!("Finished suite '{}' unsuccessfully.", identifier);
        emit::test_suite_failed(emitter, identifier).await?;
    }

    Ok(test_suite_report)
}

async fn run_test_case(
    test_case: TestCaseRun,
    emitter: &mut dyn EventEmitter<RunEvent>,
    interpreter: &Interpreter,
) -> RunResult<TestCaseReport> {

    let TestCaseRun { identifier, ty, instance, setup_class_fn, teardown_class_fn, tests } = test_case;

    let mut test_case_report = TestCaseReport {
        name: Clone::clone(&identifier),
        tests: Vec::with_capacity(tests.len()),
    };

    debug!("Running case: {}", identifier);

    emit::test_case_started(emitter, Clone::clone(&identifier)).await?;

    interpreter.enter(|vm| {
        if let Some(setup_class_fn) = &setup_class_fn
        && let Err(err) = setup_class_fn.call((ty.clone(), ), vm) {
            print_error(err);
        }
    });

    for test in tests {
        let test_report = run_test(test, Clone::clone(&instance), emitter, interpreter).await?;
        test_case_report.tests.push(test_report);
    }

    interpreter.enter(|vm| {
        if let Some(teardown_class_fn) = teardown_class_fn
        && let Err(error) = teardown_class_fn.call((ty, ), vm) {
            print_error(error);
        }
    });

    if test_case_report.is_success() {
        info!("Finished case '{}' successfully.", identifier);
        emit::test_case_passed(emitter, identifier).await?;
    }
    else {
        info!("Finished case '{}' unsuccessfully.", identifier);
        emit::test_case_failed(emitter, identifier).await?;
    }

    Ok(test_case_report)
}

async fn run_test(
    test: TestRun,
    instance: PyObjectRef,
    emitter: &mut dyn EventEmitter<RunEvent>,
    interpreter: &Interpreter,
) -> RunResult<TestReport> {

    let TestRun { identifier, test_fn, setup_fn, teardown_fn } = test;

    debug!("Running test: {}", identifier);

    emit::test_started(emitter, Clone::clone(&identifier)).await?;

    let test_report = interpreter.enter(|vm| {
        let buffer = Rc::new(RefCell::new(Some(Vec::<String>::new())));
        let stdout = make_stdout_object(vm, Rc::clone(&buffer));
        let stderr = Clone::clone(&stdout);
        let open_fn = move |file: String, mode: OptionalArg<String>, vm: &VirtualMachine| {
            FileHandler::new(file, mode, vm)
        };

        let report_properties = Rc::new(RefCell::new(Vec::new()));
        instance.set_attr("report", make_report_properties(Rc::clone(&report_properties), vm), vm)
            .map_err(|_| {
                let owner = instance.str(vm).expect("Invoke `__str__` on object");
                PythonReflectionError::new_attribute_not_writable_error(owner.to_string(), "report")
            })
            .map_err(|cause| RunError::new_python_reflection_error(Clone::clone(&identifier), cause))?;

        if let Some(setup_fn) = setup_fn
        && let Err(err) = setup_fn.call((Clone::clone(&instance), ), vm) { // TODO: Decide, what should happen when a setup function fails.
            print_error(err);
        }

        vm.sys_module.set_attr("stdout", stdout, vm)
            .map_err(|_| PythonReflectionError::new_attribute_not_writable_error("sys", "stdout"))
            .map_err(|cause| RunError::new_python_reflection_error(Clone::clone(&identifier), cause))?;
        vm.sys_module.set_attr("stderr", stderr, vm)
            .map_err(|_| PythonReflectionError::new_attribute_not_writable_error("sys", "stderr"))
            .map_err(|cause| RunError::new_python_reflection_error(Clone::clone(&identifier), cause))?;
        vm.builtins.set_attr("open", vm.new_function("open", open_fn), vm)
            .map_err(|_| PythonReflectionError::new_attribute_not_writable_error("builtins", "open"))
            .map_err(|cause| RunError::new_python_reflection_error(Clone::clone(&identifier), cause))?;

        let py_result = test_fn.call((Clone::clone(&instance), ), vm);

        let outcome = match py_result {
            Ok(_) => Outcome::Success,
            Err(error) => {
                print_error(error);
                Outcome::Failure
            },
        };

        if let Some(teardown_fn) = teardown_fn {
            teardown_fn.call((Clone::clone(&instance), ), vm).unwrap(); // TODO: Decide what should happen when a teardown function fails.
        }

        Ok(TestReport {
            identifier: Clone::clone(&identifier),
            outcome,
            output: buffer.take().expect("Buffer should not be borrowed at the end of a test"),
            properties: report_properties.take(),
        })
    });

    if let Ok(report) = &test_report {
        if report.is_success() {
            info!("Finished test '{}' successfully.", identifier);
            emit::test_passed(emitter, identifier).await?;
        }
        else {
            info!("Finished test '{}' unsuccessfully.", identifier);
            emit::test_failed(emitter, identifier).await?;
        }
    }

    test_report
}

fn print_error(error: PyBaseExceptionRef) {
    let args = error.args().iter().map(|arg| format!("{arg:?}")).collect::<Vec<String>>().join(", ");
    error!("error: {error:?}: {:?}", args);
}
