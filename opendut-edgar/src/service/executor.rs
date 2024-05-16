use tokio::process::Command;
use tracing::{debug, error, info, warn};

use opendut_types::peer;
use opendut_types::peer::executor::{ContainerCommand, ContainerName, Engine, ExecutorDescriptor};

#[tracing::instrument(skip_all)]
pub fn setup_executors(executors: Vec<peer::configuration::Parameter<ExecutorDescriptor>>) { //TODO make idempotent

    let executors = executors.into_iter()
        .filter_map(|executor| { //TODO properly handle Present vs. Absent
            if matches!(executor.target, peer::configuration::ParameterTarget::Present) {
                Some(executor.value)
            } else {
                None
            }
        });

    for executor in executors {
        match executor {
            ExecutorDescriptor::Executable => warn!("Executing Executable not yet implemented."),
            ExecutorDescriptor::Container {
                engine,
                name,
                image,
                volumes,
                devices,
                envs,
                ports,
                command,
                args
            } => {
                let engine = match engine {
                    Engine::Docker => { "docker" }
                    Engine::Podman => { "podman" }
                };
                let mut cmd = Command::new(engine);
                cmd.arg("run");
                cmd.arg("--restart=unless-stopped");
                if let ContainerName::Value(name) = name {
                    cmd.args(["--name", name.as_str()]);
                }
                for port in ports {
                    cmd.args(["--publish", port.value()]);
                }
                for volume in volumes {
                    cmd.args(["--volume", volume.value()]);
                }
                for device in devices {
                    cmd.args(["--devices", device.value()]);
                }
                for env in envs {
                    cmd.args(["--env", &format!("{}={}", env.name(), env.value())]);
                }
                cmd.arg(image.value());
                if let ContainerCommand::Value(command) = command {
                    cmd.arg(command.as_str());
                }
                for arg in args {
                    cmd.arg(arg.value());
                }
                debug!("Command: {:?}", cmd);
                match cmd.spawn() {
                    Ok(_) => { info!("Container started.") }
                    Err(_) => { error!("Failed to start container.") }
                };
            }
        }
    }
}
