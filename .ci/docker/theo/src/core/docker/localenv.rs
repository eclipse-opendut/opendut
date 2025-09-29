use anyhow::Error;
use crate::core::docker::DockerCommand;

pub(crate) fn docker_localenv_shutdown(delete_volumes: bool) -> Result<i32, Error> {
    let mut command = DockerCommand::new();
    command.add_localenv_args();
    if delete_volumes {
        command.arg("down").arg("--volumes");
    } else {
        command.arg("down");
    }
    command.expect_status("Failed to shutdown localenv services.")
}
