use std::collections::HashSet;
use std::thread::sleep;
use std::time::Duration;

use anyhow::{anyhow, Error};
use strum::Display;

use EdgarDeploymentStatus::{Provisioned, Ready};
use opendut_edgar_kernel_modules::{default_builtin_module_dir, default_module_file, required_kernel_modules};

use crate::core::{SLEEP_TIME_SECONDS, TheoError, TIMEOUT_SECONDS};
use crate::core::docker::command::DockerCommand;
use crate::core::docker::compose::{docker_compose_build, docker_compose_down};
use crate::core::docker::edgar::{edgar_container_names, EDGAR_LEADER_NAME, EDGAR_PEER_1_NAME, EDGAR_PEER_2_NAME};
use crate::core::docker::services::DockerCoreServices;
use crate::core::project::TheoDynamicEnvVars;

#[derive(clap::Parser)]
pub struct TestEdgarCli {
    #[command(subcommand)]
    pub task: TaskCli,
}

#[derive(clap::Subcommand)]
pub enum TaskCli {
    /// Start EDGAR cluster
    Start,
    /// Stop EDGAR cluster
    Stop,
    /// Build EDGAR
    Build,
}

impl TestEdgarCli {
    pub(crate) fn default_handling(&self) -> crate::Result {
        match self.task {
            TaskCli::Start => {
                load_edgar_kernel_modules()?;
                docker_compose_down(DockerCoreServices::Edgar.as_str(), false)?;
                docker_compose_build(DockerCoreServices::Edgar.as_str())?;
                start_edgar_in_docker()?;
                wait_for_all_edgar_peers_are(Provisioned)?;
                wait_for_all_edgar_peers_are(Ready)?;
                check_edgar_leader_ping_all()?;
                check_edgar_can_ping()?;
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

#[derive(Debug, Display, Clone, Copy)]
pub enum EdgarDeploymentStatus {
    Provisioned,
    Ready,
}

fn wait_for_all_edgar_peers_are(task: EdgarDeploymentStatus) -> crate::Result {
    println!("STAGE: EDGAR {}", task);
    let timeout = Duration::from_secs(TIMEOUT_SECONDS);
    let start = std::time::Instant::now();
    let container_names = edgar_container_names()?;
    let mut remaining_edgar_names: HashSet<String> = HashSet::from_iter(container_names);

    while !remaining_edgar_names.is_empty() {
        let duration = start.elapsed();
        if duration > timeout {
            return Err(anyhow!(
                TheoError::Timeout(String::from("An error occurred while waiting for the Edgar leader to deploy."))
            ));
        }

        for edgar_name in remaining_edgar_names.clone() {
            match task {
                Provisioned => {
                    if check_edgar_container_provisioning_done(&edgar_name)? {
                        println!("EDGAR peer '{}' is provisioned.", edgar_name);
                        remaining_edgar_names.remove(&edgar_name);
                    }
                }
                Ready => {
                    println!("Checking if EDGAR peer '{}' is ready...", edgar_name);
                    let leader_arg = if edgar_name.eq(EDGAR_LEADER_NAME) { "leader" } else { "peer" };
                    DockerCommand::new_exec(&edgar_name)
                        .arg("/opt/wait_until_ready.sh")
                        .arg(leader_arg)
                        .expect_status(&format!("Failed to check if EDGAR peer '{}' is ready.", edgar_name))?;
                    println!("EDGAR peer '{}' is ready.", edgar_name);
                    remaining_edgar_names.remove(&edgar_name);
                }
            }
        }
        // Print message with duration in seconds, formatted to 6 characters
        println!("{:^width$} seconds - Waiting for edgar leader to be deployed...", duration.as_secs(), width=6);
        sleep(Duration::from_secs(SLEEP_TIME_SECONDS));
    }

    Ok(())
}


fn check_edgar_container_provisioning_done(container_name: &str) -> Result<bool, Error> {
    let exists = DockerCommand::container_exists(container_name);
    if !exists {
        Err(TheoError::DockerCommandFailed(format!("Edgar container '{}' has terminated or does not exists!", container_name)).into())
    } else {
        check_edgar_provisioning_finished(container_name)
    }
}

fn check_edgar_provisioning_finished(container_name: &str) -> Result<bool, Error> {
    let command_output = DockerCommand::new_exec(container_name)
        .arg("cat")
        .arg("/opt/signal/result.txt")
        .expect_output(format!("Failed to check if edgar {} was provisioned.", container_name).as_str());
    match command_output {
        Ok(output) => {
            let status_code = output.status.code().unwrap_or(1) == 0;
            if status_code  {
                let message = String::from_utf8(output.stdout)?;
                if message.eq("Success") {
                    Ok(true)
                } else {
                    Err(anyhow!("Edgar leader provisioning failed: '{}'. Check 'docker logs edgar-leader'.", message))
                }
            } else {
                Ok(false)
            }
        }
        Err(error) => {
            Err(error)
        }
    }
}


fn check_edgar_leader_ping_all() -> Result<i32, Error> {
    println!("STAGE: EDGAR ping all");
    println!("       Checking if all EDGAR peers respond to ping...");
    DockerCommand::new_exec(EDGAR_LEADER_NAME)
        .arg("/opt/pingall.sh")
        .expect_status("Failed to check if all EDGAR peers respond to ping.")
}

fn check_edgar_can_ping() -> Result<i32, Error> {
    println!("STAGE: EDGAR CAN ping all");
    println!("       Checking if all EDGAR peers respond to CAN ping...");

    DockerCommand::new()
        .arg("exec")
        .arg("-d")
        .arg(EDGAR_PEER_1_NAME)
        .arg("python3")
        .arg("/opt/pingall_can.py")
        .arg("responder")
        .expect_status("Failed to start CAN ping responder on edgar-peer-1.")?;

    sleep(Duration::from_secs(10));

    DockerCommand::new_exec(EDGAR_PEER_2_NAME)
        .arg("python3")
        .arg("/opt/pingall_can.py")
        .arg("sender")
        .expect_status("Failed to start CAN ping sender on edgar-peer-2.")
}

fn load_edgar_kernel_modules() -> Result<(), Error> {
    for kernel_module in required_kernel_modules() {
        if !kernel_module.is_loaded(&default_module_file(), &default_builtin_module_dir())? {
            kernel_module.load()?;
        }        
    }
    Ok(())
}
