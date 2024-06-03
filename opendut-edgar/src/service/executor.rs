//use tokio::process::Command;
//use tracing::{debug, error, info, warn};
use tracing::warn;

use opendut_types::peer::{self, executor::ExecutorDescriptor};
//use opendut_types::peer::executor::{ContainerCommand, ContainerName, Engine, ExecutorDescriptor};

use super::test_execution::executor_manager::ExecutorManagerRef;

#[tracing::instrument(skip_all)]
pub fn setup_executors(executors: Vec<peer::configuration::Parameter<ExecutorDescriptor>>, executor_manager: ExecutorManagerRef) { //TODO make idempotent

    let executors = executors.into_iter()
        .filter_map(|executor| { //TODO properly handle Present vs. Absent
            if matches!(executor.target, peer::configuration::ParameterTarget::Present) {
                Some(executor.value)
            } else {
                None
            }
        });

    let mut executor_manager_unlocked = executor_manager.lock().unwrap();

    executor_manager_unlocked.terminate_executors();

    for executor in executors {
        executor_manager_unlocked.create_new_executor(executor);
        // match executor {
        //     ExecutorDescriptor::Executable => warn!("Executing Executable not yet implemented."),
        //     ExecutorDescriptor::Container {
        //         engine,
        //         name,
        //         image,
        //         volumes: _,
        //         devices: _,
        //         envs,
        //         ports: _,
        //         command,
        //         args,
        //         results_url
        //     } => {
        //         let container_config = ContainerConfiguration{
        //             name,
        //             engine,
        //             image,
        //             command,
        //             args,
        //             envs,
        //             results_url,
        //         };
        //         tokio::spawn(async move {
        //             ContainerManager::new(container_config).start().await;
        //         });

                // let engine = match engine {
                //     Engine::Docker => { "docker" }
                //     Engine::Podman => { "podman" }
                // };
                // let mut cmd = Command::new(engine);
                // cmd.arg("run");
                // cmd.arg("--restart=unless-stopped");
                // if let ContainerName::Value(name) = name {
                //     cmd.args(["--name", name.as_str()]);
                // }
                // for port in ports {
                //     cmd.args(["--publish", port.value()]);
                // }
                // for volume in volumes {
                //     cmd.args(["--volume", volume.value()]);
                // }
                // for device in devices {
                //     cmd.args(["--devices", device.value()]);
                // }
                // for env in envs {
                //     cmd.args(["--env", &format!("{}={}", env.name(), env.value())]);
                // }
                // cmd.arg(image.value());
                // if let ContainerCommand::Value(command) = command {
                //     cmd.arg(command.as_str());
                // }
                // for arg in args {
                //     cmd.arg(arg.value());
                // }
                // debug!("Command: {:?}", cmd);
                // match cmd.spawn() {
                //     Ok(_) => { info!("Container started.") }
                //     Err(_) => { error!("Failed to start container.") }
                // };
            //}
        //}
    }
}
