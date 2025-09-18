pub mod container;
pub mod file;
pub mod metadata;
pub mod parameters;
pub mod report;
pub mod unittest;

use rustpython_vm::builtins::PyModule;
use rustpython_vm::{pymodule, PyRef};

#[pymodule]
mod viper {}

pub fn create_module(vm: &rustpython_vm::vm::VirtualMachine) -> PyRef<PyModule> {
    let module = viper::make_module(vm);
    module.set_attr("container", container::container::make_module(vm), vm).unwrap();
    module.set_attr("file", file::file::make_module(vm), vm).unwrap();
    module.set_attr("metadata", metadata::metadata::make_module(vm), vm).unwrap();
    module.set_attr("parameters", parameters::parameters::make_module(vm), vm).unwrap();
    module.set_attr("report", report::report::make_module(vm), vm).unwrap();
    module.set_attr("unittest", unittest::unittest::make_module(vm), vm).unwrap();
    module
}
