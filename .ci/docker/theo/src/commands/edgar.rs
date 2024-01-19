use anyhow::Error;

use crate::core::docker::{docker_compose_build, docker_compose_down, DockerCommand, DockerCoreServices};

#[derive(Debug, clap::Parser)]
pub struct TestEdgarCli {
    #[command(subcommand)]
    pub task: TaskCli,
}

#[derive(Debug, clap::Subcommand)]
pub enum TaskCli {
    #[command(about = "Start edgar cluster.")]
    Start,
    #[command(about = "Stop edgar cluster.")]
    Stop,
    #[command(about = "Build edgar.")]
    Build,
}

impl TestEdgarCli {

    pub(crate) fn default_handling(&self) -> crate::Result {

        match self.task {
            TaskCli::Start => {
                docker_compose_build(DockerCoreServices::Edgar.as_str())?;
                start_edgar_in_docker()?;
            }
            TaskCli::Stop => {
                docker_compose_down(DockerCoreServices::Edgar.as_str(), false)?;
            }
            TaskCli::Build => {
                docker_compose_build(DockerCoreServices::Edgar.as_str())?;
            }
        }
        Ok(())
    }
}

fn start_edgar_in_docker() -> Result<i32, Error> {
    DockerCommand::new()
        .add_common_args(DockerCoreServices::Edgar.as_str())
        .add_netbird_api_key_to_env()?
        .arg("up")
        .arg("-d")
        .expect_status("Failed to start edgar cluster in docker.")
}

