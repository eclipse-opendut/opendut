use crate::runtime::compile::CompileResult;
use crate::runtime::types::compile::code::SourceCode;
use crate::runtime::types::compile::error::CompilationError;
use crate::runtime::types::py::error::{PythonReflectionError, PythonRuntimeError};
use rustpython_vm::builtins::{PyCode, PyModule, PyStr};
use rustpython_vm::scope::Scope;
use rustpython_vm::{compiler, Interpreter, PyRef, Settings, VirtualMachine};

pub fn create_interpreter() -> Interpreter {
    Interpreter::with_init(Settings::default(), |_vm| {
        // currently nothing to do.
    })
}

pub fn compile_source_code(source_code: &SourceCode, py_interpreter: &Interpreter) -> CompileResult<PyRef<PyModule>> {
    py_interpreter.enter(|vm| {
        let python_code = compile_source(source_code, vm)?;

        inject_sys_modules(vm)
            .map_err(|error| CompilationError::new_python_reflection_error(Clone::clone(&source_code.identifier), error))?;

        let module_attributes = vm.ctx.new_dict();

        vm.run_code_obj(python_code, Scope::new(None, Clone::clone(&module_attributes)))
            .map_err(PythonRuntimeError::from_base_exception)
            .map_err(|error| CompilationError::new_python_runtime_error(Clone::clone(&source_code.identifier), error))?;

        Ok(vm.new_module("example", module_attributes, None)) // TODO: Change python module name.
    })
}

fn compile_source(source_code: &SourceCode, vm: &VirtualMachine) -> CompileResult<PyRef<PyCode>> {
    vm.compile(&source_code.code, compiler::Mode::Exec, String::from("_embedded_"))
        .map_err(|error| Box::new(CompilationError::new_python_compilation_error(Clone::clone(&source_code.identifier), error.to_string())))
}

fn inject_sys_modules(vm: &VirtualMachine) -> Result<(), PythonReflectionError> {
    const MODULES_ATTRIBUTE_NAME: &str = "modules";
    const NAME_ATTRIBUTE_NAME: &str = "__name__";
    let sys_modules = vm.sys_module.get_attr(MODULES_ATTRIBUTE_NAME, vm)
        .map_err(|_| PythonReflectionError::new_no_such_attribute_error(MODULES_ATTRIBUTE_NAME))?;
    let viper_module = viper_py::create_module(vm);
    let module_name = viper_module.get_attr(NAME_ATTRIBUTE_NAME, vm)
        .map_err(|_| PythonReflectionError::new_no_such_attribute_error(NAME_ATTRIBUTE_NAME))?;
    let module_name = module_name
        .downcast_ref::<PyStr>()
        .ok_or_else(|| PythonReflectionError::new_downcast_error(&module_name, "PyStr"))?
        .to_string();
    sys_modules.set_item(module_name.as_str(), Clone::clone(&viper_module).into(), vm)
        .map_err(|_| {
            let str = sys_modules.str(vm).expect("Invoke `__str__` on module");
            PythonReflectionError::new_attribute_not_writable_error(str.to_string(), module_name)
        })
}
