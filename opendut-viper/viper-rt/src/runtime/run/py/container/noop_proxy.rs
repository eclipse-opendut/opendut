use rustpython_vm::builtins::{PyBaseExceptionRef, PyListRef};
use rustpython_vm::function::{KwArgs, OptionalArg};
use rustpython_vm::{PyObjectRef, PyResult, VirtualMachine};
use opendut_viper_py::container::container::PyContainerInspectInfo;
use opendut_viper_py::container::ContainerRuntimeProxy;

pub struct NoopProxy;

impl NoopProxy {
    fn runtime_error(message: &str, vm: &VirtualMachine) -> PyBaseExceptionRef {
        vm.new_runtime_error(format!("{message} No container runtime available!"))
    }
}

impl ContainerRuntimeProxy for NoopProxy {

    fn create_container(&self, _image: String, _command: OptionalArg<PyObjectRef>, _kwargs: KwArgs, vm: &VirtualMachine) -> PyResult<String> {
        Err(NoopProxy::runtime_error("Cannot create container!", vm))
    }

    fn start_container(&self, _container: String, vm: &VirtualMachine) -> PyResult<()> {
        Err(NoopProxy::runtime_error("Cannot start container!", vm))
    }

    fn stop_container(&self, _container: String, vm: &VirtualMachine) -> PyResult<()> {
        Err(NoopProxy::runtime_error("Cannot stop container!", vm))
    }

    fn run_container(&self, _image: String, _command: OptionalArg<PyObjectRef>, _kwargs: KwArgs, vm: &VirtualMachine) -> PyResult<String> {
        Err(NoopProxy::runtime_error("Cannot run container!", vm))
    }

    fn remove_container(&self, _container: String, vm: &VirtualMachine) -> PyResult<()> {
        Err(NoopProxy::runtime_error("Cannot remove container!", vm))
    }

    fn wait_container(&self, _container: String, vm: &VirtualMachine) -> PyResult<i64> {
        Err(NoopProxy::runtime_error("Cannot wait for container!", vm))
    }

    fn list_containers(&self, vm: &VirtualMachine) -> PyResult<PyListRef> {
        Err(NoopProxy::runtime_error("Cannot list containers!", vm))
    }

    fn inspect_container(&self, _container: String, vm: &VirtualMachine) -> PyResult<PyContainerInspectInfo> {
        Err(NoopProxy::runtime_error("Cannot inspect container!", vm))
    }

    fn log_container(&self, _container: String, vm: &VirtualMachine) -> PyResult<PyListRef> {
        Err(NoopProxy::runtime_error("Cannot log container!", vm))
    }

    fn pull_image(&self, _image: String, vm: &VirtualMachine) -> PyResult<String> {
        Err(NoopProxy::runtime_error("Cannot pull container image!", vm))
    }

    fn remove_image(&self, _image: String, vm: &VirtualMachine) -> PyResult<()> {
        Err(NoopProxy::runtime_error("Cannot remove container image!", vm))
    }

    fn list_images(&self, vm: &VirtualMachine) -> PyResult<PyListRef> {
        Err(NoopProxy::runtime_error("Cannot list container images!", vm))
    }
}
