mod real_proxy;
mod noop_proxy;

use noop_proxy::NoopProxy;
use real_proxy::RealProxy;
use rustpython_vm::{PyPayload, PyRef, VirtualMachine};
use std::rc::Rc;
use opendut_viper_containers::ContainerRuntime;
use viper_py::container::container::PyContainerRuntimeProxy;
use viper_py::container::ContainerRuntimeProxy;

pub fn make_container_runtime_proxy_object(client: Option<&ContainerRuntime>, vm: &VirtualMachine) -> PyRef<PyContainerRuntimeProxy> {
    let proxy = client
        .map(|client| Rc::new(RealProxy::new(client.to_owned())) as Rc<dyn ContainerRuntimeProxy>)
        .unwrap_or_else(|| Rc::new(NoopProxy));
    PyContainerRuntimeProxy::new(proxy, vm).into_ref(&vm.ctx)
}
