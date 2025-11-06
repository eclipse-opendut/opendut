#![allow(clippy::module_inception)]
use rustpython_vm::function::{KwArgs, OptionalArg};
use rustpython_vm::{pymodule, PyObjectRef, PyResult, VirtualMachine};
use rustpython_vm::builtins::PyListRef;
use crate::container::container::PyContainerInspectInfo;

pub trait ContainerRuntimeProxy {
    fn create_container(&self, image: String, command: OptionalArg<PyObjectRef>, kwargs: KwArgs, vm: &VirtualMachine) -> PyResult<String>;
    fn start_container(&self, container: String, vm: &VirtualMachine) -> PyResult<()>;
    fn stop_container(&self, container: String, vm: &VirtualMachine) -> PyResult<()>;
    fn run_container(&self, container: String, command: OptionalArg<PyObjectRef>, kwargs: KwArgs, vm: &VirtualMachine) -> PyResult<String>;
    fn remove_container(&self, container: String, vm: &VirtualMachine) -> PyResult<()>;
    fn wait_container(&self, container: String, vm: &VirtualMachine) -> PyResult<i64>;
    fn list_containers(&self, vm: &VirtualMachine) -> PyResult<PyListRef>;
    fn inspect_container(&self, container: String, vm: &VirtualMachine) -> PyResult<PyContainerInspectInfo>;
    fn log_container(&self, container: String, vm: &VirtualMachine) -> PyResult<PyListRef>;
    fn pull_image(&self, image: String, vm: &VirtualMachine) -> PyResult<String>;
    fn remove_image(&self, image: String, vm: &VirtualMachine) -> PyResult<()>;
    fn list_images(&self, vm: &VirtualMachine) -> PyResult<PyListRef>;
}

#[pymodule]
pub mod container {
    use crate::container::ContainerRuntimeProxy;
    use rustpython_vm::function::{KwArgs, OptionalArg};
    use rustpython_vm::{pyclass, PyObjectRef, PyPayload, PyResult, VirtualMachine};
    use std::fmt::Formatter;
    use std::rc::Rc;
    use rustpython_vm::builtins::PyListRef;

    #[pyclass(name = "ContainerRuntimeProxy", no_attr)]
    #[derive(PyPayload)]
    pub struct PyContainerRuntimeProxy {
        proxy: Rc<dyn ContainerRuntimeProxy>,
        image: PyObjectRef,
    }

    #[pyclass]
    #[opendut_viper_pygen::pygen]
    impl PyContainerRuntimeProxy {

        #[viper(skip)]
        pub fn new(proxy: Rc<dyn ContainerRuntimeProxy>, vm: &VirtualMachine) -> PyContainerRuntimeProxy {
            let image = PyContainerRuntimeImageProxy::new(Rc::clone(&proxy));
            let image = vm.new_pyobj(image);
            Self { proxy, image }
        }

        #[pymethod]
        fn create(&self, image: String, #[viper(default = "")] command: OptionalArg<PyObjectRef>, kwargs: KwArgs, #[viper(skip)] vm: &VirtualMachine) -> PyResult<String> {
            self.proxy.create_container(image, command, kwargs, vm)
        }

        #[pymethod]
        fn start(&self, name: String, #[viper(skip)] vm: &VirtualMachine) -> PyResult<()> {
            self.proxy.start_container(name, vm)
        }

        #[pymethod]
        fn stop(&self, name: String, #[viper(skip)] vm: &VirtualMachine) -> PyResult<()> {
            self.proxy.stop_container(name, vm)
        }

        #[pymethod]
        fn run(&self, image: String, #[viper(default = "")] command: OptionalArg<PyObjectRef>, kwargs: KwArgs, #[viper(skip)] vm: &VirtualMachine) -> PyResult<String> {
            self.proxy.run_container(image, command, kwargs, vm)
        }

        #[pymethod]
        fn remove(&self, container: String, #[viper(skip)] vm: &VirtualMachine) -> PyResult<()> {
            self.proxy.remove_container(container, vm)
        }

        #[pymethod]
        fn wait(&self, container: String, #[viper(skip)] vm: &VirtualMachine) -> PyResult<i64> {
            self.proxy.wait_container(container, vm)
        }

        #[pymethod]
        fn list(&self, #[viper(skip)] vm: &VirtualMachine) -> PyResult<PyListRef> {
            self.proxy.list_containers(vm)
        }

        #[pymethod]
        fn inspect(&self, name: String, vm: &VirtualMachine) -> PyResult<PyContainerInspectInfo> {
            self.proxy.inspect_container(name, vm)
        }

        #[pymethod]
        fn log(&self, name: String, vm: &VirtualMachine) -> PyResult<PyListRef> {
            self.proxy.log_container(name, vm)
        }

        #[pygetset]
        fn image(&self) -> PyObjectRef {
            Clone::clone(&self.image)
        }
    }

    impl core::fmt::Debug for PyContainerRuntimeProxy {
        fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
            formatter.debug_struct("PyContainerRuntimeProxy")
                .finish()
        }
    }

    #[pyclass(name = "ContainerRuntimeImageProxy", no_attr)]
    #[derive(PyPayload)]
    pub struct PyContainerRuntimeImageProxy {
        proxy: Rc<dyn ContainerRuntimeProxy>,
    }

    #[pyclass]
    #[opendut_viper_pygen::pygen]
    impl PyContainerRuntimeImageProxy {

        #[viper(skip)]
        pub fn new(proxy: Rc<dyn ContainerRuntimeProxy>) -> PyContainerRuntimeImageProxy {
            Self { proxy }
        }

        #[pymethod]
        fn pull(&self, image: String, #[viper(skip)] vm: &VirtualMachine) -> PyResult<String> {
            self.proxy.pull_image(image, vm)
        }

        #[pymethod]
        fn remove(&self, image: String, #[viper(skip)] vm: &VirtualMachine) -> PyResult<()> {
            self.proxy.remove_image(image, vm)
        }

        #[pymethod]
        fn list(&self, #[viper(skip)] vm: &VirtualMachine) -> PyResult<PyListRef> {
            self.proxy.list_images(vm)
        }
    }

    impl core::fmt::Debug for PyContainerRuntimeImageProxy {
        fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
            formatter.debug_struct("PyContainerRuntimeImageProxy")
                .finish()
        }
    }

    #[derive(Debug, Clone)]
    pub enum PyContainerStateStatus {
        Empty,
        Created,
        Running,
        Paused,
        Restarting,
        Removing,
        Exited,
        Dead,
    }

    impl ::std::fmt::Display for PyContainerStateStatus {
        fn fmt(&self, f: &mut Formatter) -> ::std::fmt::Result {
            match *self {
                PyContainerStateStatus::Empty => write!(f, ""),
                PyContainerStateStatus::Created => write!(f, "created"),
                PyContainerStateStatus::Running => write!(f, "running"),
                PyContainerStateStatus::Paused => write!(f, "paused"),
                PyContainerStateStatus::Restarting => write!(f, "restarting"),
                PyContainerStateStatus::Removing => write!(f, "removing"),
                PyContainerStateStatus::Exited => write!(f, "exited"),
                PyContainerStateStatus::Dead => write!(f, "dead"),
            }
        }
    }

    #[pyattr]
    #[pyclass(name="ContainerState")]
    #[derive(Debug, PyPayload, Clone)]
    pub struct PyContainerState {
        pub status: PyContainerStateStatus,
        pub running: bool,
        pub paused: bool,
        pub restarting: bool,
        pub oom_killed: bool,
        pub dead: bool,
        pub exit_code: i64,
    }

    #[pyclass]
    impl PyContainerState {
        #[pygetset]
        pub fn status(&self) -> PyResult<String> {
            Ok(self.status.to_string())
        }

        #[pygetset]
        pub fn running(&self) -> PyResult<bool> {
            Ok(self.running)
        }

        #[pygetset]
        pub fn paused(&self) -> PyResult<bool> {
            Ok(self.paused)
        }

        #[pygetset]
        pub fn restarting(&self) -> PyResult<bool> {
            Ok(self.restarting)
        }

        #[pygetset]
        pub fn oom_killed(&self) -> PyResult<bool> {
            Ok(self.oom_killed)
        }

        #[pygetset]
        pub fn dead(&self) -> PyResult<bool> {
            Ok(self.dead)
        }

        #[pygetset]
        pub fn exit_code(&self) -> PyResult<i64> {
            Ok(self.exit_code)
        }
    }

    #[pyattr]
    #[pyclass(name="ContainerInspectInfo")]
    #[derive(PyPayload, Debug)]
    pub struct PyContainerInspectInfo {
        pub id: String,
        pub name: String,
        pub created: String,
        pub path: String,
        pub args: PyListRef,
        pub state: PyContainerState,
        pub restart_count: i64,
    }

    #[pyclass]
    impl PyContainerInspectInfo {

        #[pygetset]
        pub fn id(&self) -> PyResult<String> {
            Ok(Clone::clone(&self.id))
        }

        #[pygetset]
        pub fn name(&self) -> PyResult<String> {
            Ok(Clone::clone(&self.name))
        }

        #[pygetset]
        pub fn created(&self) -> PyResult<String> {
            Ok(Clone::clone(&self.created))
        }

        #[pygetset]
        pub fn path(&self) -> PyResult<String> {
            Ok(Clone::clone(&self.path))
        }

        #[pygetset]
        pub fn args(&self) -> PyResult<PyListRef> {
            Ok(Clone::clone(&self.args))
        }

        #[pygetset]
        pub fn state(&self) -> PyResult<PyContainerState> {
            Ok(Clone::clone(&self.state))
        }

        #[pygetset]
        pub fn restart_count(&self) -> PyResult<i64> {
            Ok(self.restart_count)
        }
    }
}
