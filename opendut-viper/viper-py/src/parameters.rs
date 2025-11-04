#![allow(clippy::module_inception)]
use rustpython_vm::{pymodule, PyObjectRef, PyResult, VirtualMachine};
use rustpython_vm::builtins::{PyInt, PyStr};

pub trait ParameterValueProvider {
    fn get_value(&self, name: &str, vm: &VirtualMachine) -> PyResult<PyObjectRef>;
}

#[pymodule]
pub mod parameters {
    use crate::parameters::{parse_pybool_to_bool, parse_pyint_to_i64, parse_pyint_to_u32, parse_pystr_to_string, ParameterValueProvider};
    use rustpython_vm::builtins::{PyStr, PyTypeRef};
    use rustpython_vm::function::FuncArgs;
    use rustpython_vm::{pyclass, PyObjectRef, PyPayload, PyResult, VirtualMachine};
    use std::fmt::{Debug, Formatter};
    use std::rc::Rc;

    #[pyclass(name = "ParameterDict", no_attr)]
    #[derive(PyPayload)]
    pub struct PyParameterDict {
        provider: Rc<dyn ParameterValueProvider>
    }

    /// Provides access to the parameter values of a test.
    #[pyclass]
    #[viper_pygen::pygen]
    impl PyParameterDict {

        #[viper(skip)]
        pub fn new(provider: Rc<dyn ParameterValueProvider>) -> PyParameterDict {
            Self { provider }
        }

        /// Retrieves the value of the parameter denoted by the specified key.
        ///
        /// # Example
        ///
        /// Given the following parameter declaration:
        /// ```python
        /// DEFAULT_PORT = parameters.NumberParameter("port", default=8000)
        /// ```
        ///
        /// The parameter value can be retrieved by its descriptor:
        /// ```python
        /// print("Parameter DEFAULT_PORT: ", self.parameters.get(DEFAULT_PORT))
        /// ```
        ///
        /// Or by its name:
        /// ```python
        /// print("Parameter DEFAULT_PORT: ", self.parameters.get("port"))
        /// ```
        #[pymethod]
        fn get(&self, key: PyObjectRef, #[viper(skip)] vm: &VirtualMachine) -> PyResult<PyObjectRef> {
            let name = if let Some(name) = key.payload::<PyStr>() {
                name.as_str()
            }
            else if let Some(parameter) = key.payload::<PyBooleanParameterDescriptor>() {
                parameter.name.as_str()
            }
            else if let Some(parameter) = key.payload::<PyNumberParameterDescriptor>() {
                parameter.name.as_str()
            }
            else if let Some(parameter) = key.payload::<PyTextParameterDescriptor>() {
                parameter.name.as_str()
            }
            else {
                return Err(vm.new_type_error(String::from("Invalid parameter type")));
            };
            self.provider.get_value(name, vm)
        }
    }

    impl Debug for PyParameterDict {
        fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
            formatter.debug_struct("ParameterDict")
                .finish()
        }
    }

    #[pyattr]
    #[pyclass(name = "BooleanParameter")]
    #[derive(Debug, PyPayload)]
    pub struct PyBooleanParameterDescriptor {
        pub name: String,
        pub default: Option<bool>,
        pub display_name: Option<String>,
        pub description: Option<String>,
    }

    #[pyclass]
    impl PyBooleanParameterDescriptor {

        #[pyslot]
        fn slot_new(_cls: PyTypeRef, mut args: FuncArgs, vm: &VirtualMachine) -> PyResult {
            let name = args.take_positional()
                .ok_or_else(|| vm.new_value_error(String::from("No name argument")))
                .and_then(|arg| {
                    arg.downcast_ref::<PyStr>().map(|x| x.to_string())
                        .ok_or_else(|| vm.new_type_error(String::from("argument 0 must be a string")))
                })?;

            let mut default: Option<bool> = None;
            let mut display_name: Option<String> = None;
            let mut description: Option<String> = None;

            for (key, value) in args.kwargs {
                match key.as_str() {
                    // TODO: Typecheck does not work properly
                    "default" => {
                        default = Some(parse_pybool_to_bool(value, vm)
                            .expect("downcast to `PyBool` for `PyBoolParameterDescriptor.default`"));
                    }
                    "display_name" => {
                        display_name = Some(parse_pystr_to_string(value, vm)
                            .expect("downcast to `PyStr` for `PyTextParameterDescriptor.display_name`")
                        );
                    }
                    "description" => {
                        description = Some(parse_pystr_to_string(value, vm)
                            .expect("downcast to `PyStr` for `PyTextParameterDescriptor.description`")
                        );
                    }
                    _ => {}
                }
            }

            Ok(vm.new_pyobj(Self { name, default, display_name, description }))
        }
    }

    #[pyattr]
    #[pyclass(name = "NumberParameter")]
    #[derive(Debug, PyPayload)]
    pub struct PyNumberParameterDescriptor {
        pub name: String,
        pub default: Option<i64>,
        pub min: i64,
        pub max: i64,
        pub display_name: Option<String>,
        pub description: Option<String>,
    }

    #[pyclass]
    impl PyNumberParameterDescriptor {

        #[pyslot]
        fn slot_new(_cls: PyTypeRef, mut args: FuncArgs, vm: &VirtualMachine) -> PyResult {
            let name = args.take_positional()
                .ok_or_else(|| vm.new_value_error(String::from("No name argument")))
                .and_then(|arg| {
                    arg.downcast_ref::<PyStr>().map(|x| x.to_string())
                        .ok_or_else(|| vm.new_type_error(String::from("argument 0 must be a string")))
                })?;

            let mut default: Option<i64> = None;
            let mut min = i64::MIN;
            let mut max = i64::MAX;
            let mut display_name: Option<String> = None;
            let mut description: Option<String> = None;

            for (key, value) in args.kwargs {
                match key.as_str() {
                    "default" => {
                        default = Some(parse_pyint_to_i64(value, vm)
                            .expect("downcast to `PyInt` for `PyNumberParameterDescriptor.min`"));
                    }
                    "min" => {
                        min = parse_pyint_to_i64(value, vm)
                            .expect("downcast to `PyInt` for `PyNumberParameterDescriptor.min`");
                    }
                    "max" => {
                        max = parse_pyint_to_i64(value, vm)
                            .expect("downcast to `PyInt` for `PyNumberParameterDescriptor.max`");
                    }
                    "display_name" => {
                        display_name = Some(parse_pystr_to_string(value, vm)
                            .expect("downcast to `PyStr` for `PyNumberParameterDescriptor.display_name`")
                        );
                    }
                    "description" => {
                        description = Some(parse_pystr_to_string(value, vm)
                            .expect("downcast to `PyStr` for `PyNumberParameterDescriptor.description`")
                        );
                    }
                    _ => {}
                }
            }
            
            Ok(vm.new_pyobj(Self { name, default, min, max, display_name, description }))
        }
    }

    #[pyattr]
    #[pyclass(name = "TextParameter")]
    #[derive(Debug, PyPayload)]
    pub struct PyTextParameterDescriptor {
        pub name: String,
        pub default: Option<String>,
        pub max: u32,
        pub display_name: Option<String>,
        pub description: Option<String>,
    }

    #[pyclass]
    impl PyTextParameterDescriptor {

        #[pyslot]
        fn slot_new(_cls: PyTypeRef, mut args: FuncArgs, vm: &VirtualMachine) -> PyResult {
            let name = args.take_positional()
                .ok_or_else(|| vm.new_value_error(String::from("No name argument")))
                .and_then(|arg| {
                    arg.downcast_ref::<PyStr>().map(|x| x.to_string())
                        .ok_or_else(|| vm.new_type_error(String::from("argument 0 must be a string")))
                })?;

            let mut default: Option<String> = None;
            let mut max = u32::MAX;
            let mut display_name: Option<String> = None;
            let mut description: Option<String> = None;

            for (key, value) in args.kwargs {
                match key.as_str() {
                    "default" => {
                        default = Some(parse_pystr_to_string(value, vm)
                            .expect("downcast to `PyInt` for `PyTextParameterDescriptor.default`"));
                    }
                    "max" => {
                        max = parse_pyint_to_u32(value, vm)
                            .expect("downcast to `PyInt` for `PyTextParameterDescriptor.max`");
                    }
                    "display_name" => {
                        display_name = Some(parse_pystr_to_string(value, vm)
                            .expect("downcast to `PyStr` for `PyTextParameterDescriptor.display_name`")
                        );
                    }
                    "description" => {
                        description = Some(parse_pystr_to_string(value, vm)
                            .expect("downcast to `PyStr` for `PyTextParameterDescriptor.description`")
                        );
                    }
                    _ => {}
                }
            }
            Ok(vm.new_pyobj(Self { name, default, max, display_name, description }))
        }
    }

    #[pyattr]
    #[pyclass(name = "PeerInterfaceParameter")]
    #[derive(Debug, PyPayload)]
    pub struct PyPeerInterfaceParameterDescriptor {
    }

    #[pyclass]
    impl PyPeerInterfaceParameterDescriptor {

        #[pyslot]
        fn slot_new(_cls: PyTypeRef, _args: FuncArgs, vm: &VirtualMachine) -> PyResult {
            Ok(vm.new_pyobj(Self {} ))
        }
    }
}

fn parse_pystr_to_string(value: PyObjectRef, vm: &VirtualMachine) -> PyResult<String> {
    if let Some(text) = value.downcast_ref::<PyStr>() {
        Ok(text.to_string())
    } else {
        Err(vm.new_type_error(String::from("downcast from `PyObjectRef` to `PyStr`")))
    }
}

fn parse_pyint_to_u32(value: PyObjectRef, vm: &VirtualMachine) -> PyResult<u32> {
    if let Some(pyint) = value.downcast_ref::<PyInt>() {
        let bigint = pyint.as_bigint();
        let number = u32::try_from(bigint)
            .expect("downcast from `BigInt` to `u32`");
        Ok(number)
    } else {
        Err(vm.new_type_error(String::from("downcast from `PyObjectRef` to `PyInt`")))
    }
}

fn parse_pyint_to_i64(value: PyObjectRef, vm: &VirtualMachine) -> PyResult<i64> {
    if let Some(pyint) = value.downcast_ref::<PyInt>() {
        let bigint = pyint.as_bigint();
        let number = i64::try_from(bigint)
            .expect("downcast from `BigInt` to `i64`");
        Ok(number)
    } else {
        Err(vm.new_type_error(String::from("downcast from `PyObjectRef` to `PyInt`")))
    }
}

fn parse_pybool_to_bool(value: PyObjectRef, vm: &VirtualMachine) -> PyResult<bool> {
    if let Ok(b) = value.try_to_bool(vm) {
        Ok(b)
    } else {
        Err(vm.new_type_error(String::from("downcast from `PyObjectRef` to `PyBool`")))
    }
}
