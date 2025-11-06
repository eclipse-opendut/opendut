use rustpython_vm::builtins::{PyList, PyListRef, PyStr};
use rustpython_vm::function::OptionalArg::Present;
use rustpython_vm::function::{KwArgs, OptionalArg};
use rustpython_vm::{PyObjectRef, PyResult, VirtualMachine};
use opendut_viper_containers::{ContainerRuntime, ContainerRuntimeError, ContainerStateStatus, CreateContainerConfig, VolumeMountConfig};
use opendut_viper_py::container::container::{PyContainerInspectInfo, PyContainerState, PyContainerStateStatus};
use opendut_viper_py::container::ContainerRuntimeProxy;

pub struct RealProxy {
    client: ContainerRuntime
}

impl RealProxy {
    pub fn new(client: ContainerRuntime) -> Self {
        Self { client }
    }
}

impl ContainerRuntimeProxy for RealProxy {

    fn create_container(&self, image: String, command: OptionalArg<PyObjectRef>, kwargs: KwArgs, vm: &VirtualMachine) -> PyResult<String> {

        let container_config = get_create_container_config(image, command, kwargs, vm);
        
        futures::executor::block_on(async {
            self.client.create_container(container_config).await
        }).map_err(|err| vm.new_runtime_error(err.to_string()))
    }

    fn start_container(&self, container: String, vm: &VirtualMachine) -> PyResult<()> {
        futures::executor::block_on(async {
            self.client.start_container(container).await
        }).map_err(|err| vm.new_runtime_error(err.to_string()))
    }

    fn stop_container(&self, container: String, vm: &VirtualMachine) -> PyResult<()> {
        futures::executor::block_on(async {
            self.client.stop_container(container).await
        }).map_err(|err| vm.new_runtime_error(err.to_string()))
    }

    fn run_container(
        &self,
        image: String,
        command: OptionalArg<PyObjectRef>,
        kwargs: KwArgs,
        vm: &VirtualMachine,
    ) -> PyResult<String> {
        
        let container_config = get_create_container_config(image, command, kwargs, vm);
        
        futures::executor::block_on(async {
            self.client.run_container(container_config).await
        }).map_err(|err| vm.new_runtime_error(err.to_string()))
    }

    fn remove_container(&self, container: String, vm: &VirtualMachine) -> PyResult<()> {
        futures::executor::block_on(async {
            self.client.remove_container(container).await
        }).map_err(|err| vm.new_runtime_error(err.to_string()))
    }

    fn wait_container(&self, container: String, vm: &VirtualMachine) -> PyResult<i64> {
        futures::executor::block_on(async {
            self.client.wait_container(container).await
        }).map_err(|err| vm.new_runtime_error(err.to_string()))
    }

    fn list_containers(&self, vm: &VirtualMachine) -> PyResult<PyListRef> {
        let result = futures::executor::block_on(async {
            self.client.list_containers().await
        });
        parse_vecstring_to_pylistref(result, vm)
    }

    fn inspect_container(&self, container: String, vm: &VirtualMachine) -> PyResult<PyContainerInspectInfo> {
        let inspected_container = futures::executor::block_on(async {
            self.client.inspect_container(container).await
        }).map_err(|err| vm.new_runtime_error(err.to_string()))?;

        let state = inspected_container.state;

        let py_state = PyContainerState {
            status: get_py_container_state_status(state.status),
            running: state.running,
            paused: state.paused,
            restarting: state.restarting,
            oom_killed: state.oom_killed,
            dead: state.dead,
            exit_code: state.exit_code,
        };

        let py_args = vm.ctx.new_list(
            inspected_container.args.clone()
                .into_iter()
                .map(|s| vm.ctx.new_str(s).into())
                .collect()
        );

        Ok(
            PyContainerInspectInfo {
                id: inspected_container.id,
                name: inspected_container.name,
                created: inspected_container.created,
                path: inspected_container.path,
                args: py_args,
                state: py_state,
                restart_count: inspected_container.restart_count,
            }
        )
    }

    fn log_container(&self, container: String, vm: &VirtualMachine) -> PyResult<PyListRef> {
        let result = futures::executor::block_on(async {
            self.client.log_container(container).await
        });

        parse_vecstring_to_pylistref(result, vm)
    }

    fn pull_image(&self, image: String, vm: &VirtualMachine) -> PyResult<String> {
        futures::executor::block_on(async {
            self.client.pull_image(image).await
        }).map_err(|err| vm.new_runtime_error(err.to_string()))
    }

    fn remove_image(&self, image: String, vm: &VirtualMachine) -> PyResult<()> {
        futures::executor::block_on(async {
            self.client.remove_image(image).await
        }).map_err(|err| vm.new_runtime_error(err.to_string()))
    }

    fn list_images(&self, vm: &VirtualMachine) -> PyResult<PyListRef> {
        let result = futures::executor::block_on(async {
            self.client.list_images().await
        });

        parse_vecstring_to_pylistref(result, vm)
    }
}

fn get_create_container_config(image: String, command: OptionalArg<PyObjectRef>, kwargs: KwArgs, vm: &VirtualMachine) -> CreateContainerConfig {
    let mut name: Option<String> = None;
    let mut entrypoint: Option<Vec<String>> = None;
    let mut env: Option<Vec<String>> = None;
    let mut tty: Option<bool> = None;
    let mut open_stdin: Option<bool> = None;
    let mut user: Option<String> = None;
    let mut volumes: Vec<VolumeMountConfig> = Vec::new();
    let mut network: Option<String> = None;

    for (key, value) in kwargs {
        match key.as_str() {
            "name" => {
                name = Some(value.downcast_ref::<PyStr>()
                    .expect("downcast to `PyStr` for `CreateContainerConfig.name`").to_string())
            }
            "entrypoint" => {
                entrypoint = parse_pyobjectref_to_vec(&value, vm)
                    .expect("parsing from `PyObjectRef` to `Vec<String>` for `CreateContainerConfig.entrypoint`")
            }
            "env" => {
                env = parse_pyobjectref_to_vec(&value, vm)
                    .expect("parsing from `PyObjectRef` to `Vec<String>` for `CreateContainerConfig.env`")
            }
            "tty" => {
                tty = Some(value.is_true(vm)
                    .expect("parsing from `PyObjectRef` to `bool` for `CreateContainerConfig.tty`"))
            }
            "open_stdin" => {
                open_stdin = Some(value.is_true(vm)
                    .expect("parsing from `PyObjectRef` to `bool` for `CreateContainerConfig.open_stdin`"))
            }
            "user" => {
                user = Some(value.downcast_ref::<PyStr>()
                    .expect("downcast to `PyStr` for `CreateContainerConfig.user`").to_string())
            }
            "volumes" => {
                if let Some(volumes_strings) = parse_pyobjectref_to_vec(&value, vm)
                    .expect("parsing from `PyObjectRef` to `Vec<String>` for `CreateContainerConfig.volumes") {
                    let mounts = volumes_strings.into_iter().map(|volume_strings| {
                        let mut split = volume_strings.split(":")
                            .map(|element| element.to_owned());

                        VolumeMountConfig {
                            src: split.next().expect("Expected source for VolumeMountConfig"),
                            dst: split.next().expect("Expected destination for VolumeMountConfig"),
                        }
                    });
                    volumes.extend(mounts);
                }
            }
            "network" => {
                network = Some(value.downcast_ref::<PyStr>()
                    .expect("downcast to `PyStr` for `CreateContainerConfig.network`").to_string())
            }
            _ => {}
        }
    }

    let command = if let Present(command) = command {
        parse_pyobjectref_to_vec(&command, vm).unwrap_or_default()
    } else { None };

    CreateContainerConfig {
        name,
        image,
        command,
        entrypoint,
        env,
        tty,
        open_stdin,
        user,
        volumes,
        network,
    }
}

fn parse_vecstring_to_pylistref(result: Result<Vec<String>, ContainerRuntimeError>, vm: &VirtualMachine) -> PyResult<PyListRef> {
    match result {
        Ok(list) => {
            let py_object: Vec<PyObjectRef> = list.into_iter()
                .map(|image| vm.ctx.new_str(image.to_string()).into())
                .collect();

            Ok(vm.ctx.new_list(py_object).to_owned())
        }
        Err(err) => {
            Err(vm.new_runtime_error(err.to_string()))
        }
    }
}

fn parse_pyobjectref_to_vec(obj: &PyObjectRef, vm: &VirtualMachine) -> PyResult<Option<Vec<String>>> {

    if let Ok(pystr) = obj.clone().downcast::<PyStr>() {
        let value = pystr.to_string();
        Ok(Some(vec![value]))
    } else if let Ok(pylist) = obj.clone().downcast::<PyList>() {
        let pyobject_list = pylist.borrow_vec_mut().clone();
        let string_list = pyobject_list.iter().map(|item| item.clone().downcast::<PyStr>().unwrap().to_string()).collect();
        Ok(Some(string_list))
    } else {
        Err(vm.new_runtime_error(String::from("Error while parsing from `PyObjectRef` to `Vec<String>`")))
    }
}

fn get_py_container_state_status(variant: ContainerStateStatus) -> PyContainerStateStatus {

    match variant{
        ContainerStateStatus::Empty => PyContainerStateStatus::Empty,
        ContainerStateStatus::Created => PyContainerStateStatus::Created,
        ContainerStateStatus::Running => PyContainerStateStatus::Running,
        ContainerStateStatus::Paused => PyContainerStateStatus::Paused,
        ContainerStateStatus::Restarting => PyContainerStateStatus::Restarting,
        ContainerStateStatus::Removing => PyContainerStateStatus::Removing,
        ContainerStateStatus::Exited => PyContainerStateStatus::Exited,
        ContainerStateStatus::Dead => PyContainerStateStatus::Dead,
    }
}
