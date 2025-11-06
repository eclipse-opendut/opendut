#![allow(clippy::module_inception)]
use rustpython_vm::pymodule;

#[pymodule]
pub mod metadata {
    use rustpython_vm::builtins::PyTypeRef;
    use rustpython_vm::function::FuncArgs;
    use rustpython_vm::{pyclass, PyObjectRef, PyPayload, PyResult, VirtualMachine};

    #[pyattr]
    #[pyclass(name = "Metadata", no_attr)]
    #[derive(Debug, PyPayload)]
    pub struct PyMetadata {
        pub attributes: indexmap::IndexMap<String, PyObjectRef>,
    }

    #[pyclass]
    #[opendut_viper_pygen::pygen]
    impl PyMetadata {
        
        #[pyslot]
        #[viper(skip)]
        fn slot_new(_cls: PyTypeRef, args: FuncArgs, vm: &VirtualMachine) -> PyResult {
            if !args.args.is_empty() {
                return Err(vm.new_value_error(String::from("Metadata only supports keyworded arguments!")));
            }
            Ok(vm.new_pyobj(Self { attributes: args.kwargs }))
        }
    }
}
