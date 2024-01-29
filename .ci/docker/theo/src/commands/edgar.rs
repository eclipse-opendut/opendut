use std::thread::sleep;
use std::time::Duration;

use anyhow::{anyhow, Error};

use crate::core::{SLEEP_TIME_SECONDS, TheoError, TIMEOUT_SECONDS};
use crate::core::docker::{DockerCommand, DockerCoreServices};
use crate::core::docker::compose::{docker_compose_build, docker_compose_down};
use crate::core::project::TheoDynamicEnvVars;

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
                docker_compose_down(DockerCoreServices::Edgar.as_str(), false)?;
                docker_compose_build(DockerCoreServices::Edgar.as_str())?;
                start_edgar_in_docker()?;
                wait_for_edgar_leader_provisioned()?;
                println!("Edgar leader is provisioned. Checking if all peers respond to ping...");
                check_edgar_leader_ping_all()?;
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
    println!("Starting edgar cluster '{}' in docker.", std::env::var(TheoDynamicEnvVars::OpendutEdgarClusterName.to_string()).unwrap_or_else(|_| "edgar".to_string()));
    DockerCommand::new()
        .add_common_args(DockerCoreServices::Edgar.as_str())
        .add_netbird_api_key_to_env()?
        .arg("up")
        .arg("-d")
        .expect_status("Failed to start edgar cluster in docker.")
}


fn wait_for_edgar_leader_provisioned() -> crate::Result {
    let timeout = Duration::from_secs(TIMEOUT_SECONDS);
    let start = std::time::Instant::now();
    let mut edgar_logs = check_edgar_leader_logs()?;
    let mut first_check = true;

    while !check_edgar_leader_done()? {
        let duration = start.elapsed();
        if duration > timeout {
            return Err(anyhow!(
                TheoError::Timeout(String::from("An error occurred while waiting for the Edgar leader to deploy."))
            ));
        }
        let new_edgar_logs = check_edgar_leader_logs()?;
        if first_check {
            first_check = false;
        } else if new_edgar_logs.chars().count() == edgar_logs.chars().count() {
            println!("No progress in the logs. Check 'docker logs edgar-leader'.");
        }
        edgar_logs = new_edgar_logs.to_string();
        println!("{:^width$} seconds - Waiting for edgar leader to be deployed...", duration.as_secs(), width=6);
        sleep(Duration::from_secs(SLEEP_TIME_SECONDS));
    }
    Ok(())
}

fn check_edgar_leader_logs() -> Result<String, Error> {
    let command_output = DockerCommand::new()
        .arg("logs")
        .arg("edgar-leader")
        .expect_output("Failed to get edgar leader logs.")?;
    let output = String::from_utf8(command_output.stdout)?;
    Ok(output)
}

fn check_edgar_leader_done() -> Result<bool, Error> {
    let exists = DockerCommand::exists("edgar-leader");
    if !exists {
        Err(TheoError::DockerCommandFailed("Edgar leader container has terminated or does not exists!".to_string()).into())
    } else {
        check_edgar_leader_provisioning_finished()
    }
}

fn check_edgar_leader_provisioning_finished() -> Result<bool, Error> {
    let command_output = DockerCommand::new_exec("edgar-leader")
        .arg("ls")
        .arg("/opt/signal/success.txt")
        .expect_output("Failed to check if edgar leader was provisioned.");
    DockerCommand::check_output_status(command_output)
}


fn check_edgar_leader_ping_all() -> Result<i32, Error> {
    DockerCommand::new_exec("edgar-leader")
        .arg("/opt/pingall.sh")
        .expect_status("Failed to check if all EDGAR peers respond to ping.")
}
