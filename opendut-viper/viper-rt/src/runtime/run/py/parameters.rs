use crate::compile::ParameterName;
use crate::run::{BindingValue, Complete, ParameterBindings};
use rustpython_vm::{PyObjectRef, PyPayload, PyRef, PyResult, VirtualMachine};
use std::rc::Rc;
use opendut_viper_py::parameters::parameters::PyParameterDict;
use opendut_viper_py::parameters::ParameterValueProvider;

pub fn make_parameters_dict(bindings: Rc<ParameterBindings<Complete>>, vm: &VirtualMachine) -> PyRef<PyParameterDict> {
    PyParameterDict::new(bindings).into_ref(&vm.ctx)
}

impl ParameterValueProvider for ParameterBindings<Complete> {
    fn get_value(&self, name: &str, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
        let name = ParameterName::try_from(name)
            .map_err(|error| vm.new_lookup_error(format!("Invalid parameter name: {}", error.value)))?;
        match self.get_value(&name) {
            BindingValue::BooleanValue(value) => Ok(vm.new_pyobj(value)),
            BindingValue::NumberValue(value) => Ok(vm.new_pyobj(value)),
            BindingValue::TextValue(value) => Ok(vm.new_pyobj(value)),
        }
    }
}
