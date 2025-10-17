use std::thread::sleep;
use std::time::Duration;

use anyhow::{anyhow, Error};
use tracing::debug;
use opendut_edgar_kernel_modules::{default_builtin_module_dir, default_module_file, required_can_kernel_modules};
use status::EdgarDeploymentStatus;
use crate::core::TheoError;
use crate::core::docker::command::DockerCommand;
use crate::core::docker::compose::{docker_compose_build, docker_compose_down};
use crate::core::docker::edgar::{edgar_container_names, EDGAR_LEADER_NAME, EDGAR_PEER_1_NAME, EDGAR_PEER_2_NAME};
use crate::core::docker::services::DockerCoreServices;

mod status;


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
                println!("Stopping if EDGAR cluster is already running...");
                stop_if_running()?;

                load_edgar_kernel_modules()?;
                docker_compose_build(DockerCoreServices::Edgar.as_str())?;
                start_edgar_in_docker()?;
                status::wait_until_all_edgar_peers_are(EdgarDeploymentStatus::Provisioned)?;
                status::wait_until_all_edgar_peers_are(EdgarDeploymentStatus::Ready)?;

                // this is a workaround to ensure the bridge ip is set
                set_dut_bridge_ip_address_for_pinging()?;
                check_edgar_leader_ping_all()?;
                check_edgar_ping_can()?;
            }
            TaskCli::Stop => stop_if_running()?,
            TaskCli::Build => {
                docker_compose_build(DockerCoreServices::Edgar.as_str())?;
            }
        }
        Ok(())
    }
}

fn load_edgar_kernel_modules() -> Result<(), Error> {
    for kernel_module in required_can_kernel_modules() {
        if !kernel_module.is_loaded(&default_module_file(), &default_builtin_module_dir())? {
            kernel_module.load()?;
        }
    }
    Ok(())
}

fn start_edgar_in_docker() -> Result<i32, Error> {
    debug!("Starting EDGAR cluster in docker.");
    let mut command = DockerCommand::new();
    command
        .add_common_args(DockerCoreServices::Edgar.as_str())
        .add_localenv_secrets_args()
        .arg("up")
        .arg("-d")
        .expect_show_status("Failed to start EDGAR cluster in Docker.")
}

fn check_edgar_container_provisioning_done(container_name: &str) -> Result<bool, Error> {
    let exists = DockerCommand::container_exists(container_name);
    if !exists {
        Err(TheoError::DockerCommandFailed(format!("EDGAR container '{container_name}' has terminated or does not exists!")).into())
    } else {
        check_edgar_provisioning_finished(container_name)
    }
}

fn check_edgar_provisioning_finished(container_name: &str) -> Result<bool, Error> {
    let command_output = DockerCommand::new_exec(container_name)
        .arg("cat")
        .arg("/opt/signal/result.txt")
        .expect_output(format!("Failed to check if EDGAR {container_name} was provisioned.").as_str());
    match command_output {
        Ok(output) => {
            let status_code = output.status.code().unwrap_or(1) == 0;
            if status_code  {
                let message = String::from_utf8(output.stdout)?;
                if message.eq("Success") {
                    Ok(true)
                } else {
                    Err(anyhow!("EDGAR provisioning failed for peer: {}. Check 'docker logs {}'.", message, container_name))
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

fn set_dut_bridge_ip_address_for_pinging() -> Result<(), Error> {
    let edgar_names = edgar_container_names()?;
    
    for edgar_name in edgar_names.clone() {
        DockerCommand::new_exec(&edgar_name)
            .arg("/opt/set-dut-local-ip-address.sh")
            .expect_show_status("Failed to set dut bridge ip for EDGAR.")
            .map_err(|err| anyhow!("Failed to set dut bridge ip for EDGAR {}. Error: {}", edgar_name, err))?;
    }
    Ok(())
}

fn check_edgar_leader_ping_all() -> Result<i32, Error> {
    println!("STAGE: EDGAR ping all");
    println!("       Checking if all EDGAR peers respond to ping...");
    DockerCommand::new_exec(EDGAR_LEADER_NAME)
        .arg("/opt/pingall.sh")
        .expect_show_status("Failed to check if all EDGAR peers respond to ping.")
}

fn check_edgar_ping_can() -> Result<i32, Error> {
    println!("STAGE: EDGAR CAN ping all");
    println!("       Checking if all EDGAR peers respond to CAN ping...");

    DockerCommand::new()
        .arg("exec")
        .arg("-d")
        .arg(EDGAR_PEER_1_NAME)
        .arg("python3")
        .arg("/opt/pingall_can.py")
        .arg("responder")
        .expect_show_status("Failed to start CAN ping responder on edgar-peer-1.")?;

    sleep(Duration::from_secs(10));

    DockerCommand::new_exec(EDGAR_PEER_2_NAME)
        .arg("python3")
        .arg("/opt/pingall_can.py")
        .arg("sender")
        .expect_show_status("Failed to start CAN ping sender on edgar-peer-2.")
}

fn stop_if_running() -> crate::Result {
    docker_compose_down(DockerCoreServices::Edgar.as_str(), false)?;
    delete_deployment_and_peers()?;
    Ok(())
}

fn delete_deployment_and_peers() -> Result<i32, Error> {
    println!("STAGE: Delete deployment and cleanup");
    DockerCommand::new()
        .add_common_args(DockerCoreServices::Edgar.as_str())
        .add_localenv_secrets_args()
        .arg("run")
        .arg("--name=edgar-cleanup")
        .arg("--rm")
        .arg("leader")
        .arg("/opt/delete_deployment.sh")
        .expect_show_status("Failed to start EDGAR Cleanup container in Docker.")
}
