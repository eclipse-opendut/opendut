use std::fmt::Formatter;
use std::thread::sleep;
use std::time::Duration;

use anyhow::{anyhow, Error};

use EdgarDeploymentStatus::{Provisioned, Ready};
use opendut_edgar_kernel_modules::{default_builtin_module_dir, default_module_file, required_kernel_modules};

use crate::core::{SLEEP_TIME_SECONDS, TheoError, TIMEOUT_SECONDS};
use crate::core::docker::command::DockerCommand;
use crate::core::docker::compose::{docker_compose_build, docker_compose_down};
use crate::core::docker::edgar::{edgar_container_names, format_remaining_edgars_string, EDGAR_LEADER_NAME, EDGAR_PEER_1_NAME, EDGAR_PEER_2_NAME};
use crate::core::docker::services::DockerCoreServices;

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
                wait_until_all_edgar_peers_are(Provisioned)?;
                wait_until_all_edgar_peers_are(Ready)?;
                
                // this is a workaround to ensure the bridge ip is set
                set_dut_bridge_ip_address_for_pinging()?;
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
    println!("Starting EDGAR cluster in docker.");
    DockerCommand::new()
        .add_common_args(DockerCoreServices::Edgar.as_str())
        .add_netbird_api_key_to_env()?
        .arg("up")
        .arg("-d")
        .expect_status("Failed to start EDGAR cluster in Docker.")
}

#[derive(Debug, Clone, Copy)]
pub enum EdgarDeploymentStatus {
    Provisioned,
    Ready,
}
impl std::fmt::Display for EdgarDeploymentStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let lower_case_debug = format!("{self:?}").to_lowercase();
        write!(f, "{lower_case_debug}")
    }
}

fn wait_until_all_edgar_peers_are(target_status: EdgarDeploymentStatus) -> crate::Result {
    println!("STAGE: EDGAR {}", target_status);

    let timeout = Duration::from_secs(TIMEOUT_SECONDS);
    let start = std::time::Instant::now();
    let mut remaining_edgar_names = edgar_container_names()?;

    while !remaining_edgar_names.is_empty() {
        let remaining_edgars_string = format_remaining_edgars_string(&remaining_edgar_names);

        let duration = start.elapsed();
        if duration > timeout {
            return Err(anyhow!(
                TheoError::Timeout(format!("EDGAR cluster did not become {target_status} within {timeout:?}. Incomplete peers: {remaining_edgars_string}"))
            ));
        }

        for edgar_name in remaining_edgar_names.clone() {
            match target_status {
                Provisioned => {
                    if check_edgar_container_provisioning_done(&edgar_name)? {
                        println!("EDGAR peer '{}' is provisioned.", edgar_name);
                        remaining_edgar_names.remove(&edgar_name);
                    }
                }
                Ready => {
                    println!("Checking if EDGAR peer '{}' is ready...", edgar_name);
                    let leader_arg = if edgar_name == EDGAR_LEADER_NAME { "leader" } else { "peer" };
                    DockerCommand::new_exec(&edgar_name)
                        .arg("/opt/wait_until_ready.sh")
                        .arg(leader_arg)
                        .expect_status(&format!("Failed to check if EDGAR peer '{}' is ready.", edgar_name))?;
                    println!("EDGAR peer '{}' is ready.", edgar_name);
                    remaining_edgar_names.remove(&edgar_name);
                }
            }
        }
        // Print message with duration in seconds, formatted to 4 characters
        println!("{:>width$} seconds - Waiting for peers to be {target_status}: {remaining_edgars_string}", duration.as_secs(), width=4);
        sleep(Duration::from_secs(SLEEP_TIME_SECONDS));
    }

    Ok(())
}

fn check_edgar_container_provisioning_done(container_name: &str) -> Result<bool, Error> {
    let exists = DockerCommand::container_exists(container_name);
    if !exists {
        Err(TheoError::DockerCommandFailed(format!("EDGAR container '{}' has terminated or does not exists!", container_name)).into())
    } else {
        check_edgar_provisioning_finished(container_name)
    }
}

fn check_edgar_provisioning_finished(container_name: &str) -> Result<bool, Error> {
    let command_output = DockerCommand::new_exec(container_name)
        .arg("cat")
        .arg("/opt/signal/result.txt")
        .expect_output(format!("Failed to check if EDGAR {} was provisioned.", container_name).as_str());
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
            .arg("/opt/set-br-opendut-ip-address.sh")
            .expect_status("Failed to set dut bridge ip for EDGAR.")
            .map_err(|err| anyhow!("Failed to set dut bridge ip for EDGAR {}. Error: {}", edgar_name, err))?;
    }
    Ok(())
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
