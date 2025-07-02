use std::fmt::Formatter;
use std::time::Duration;
use anyhow::anyhow;
use std::thread::sleep;
use crate::commands::edgar;
use crate::core::docker::command::DockerCommand;
use crate::core::docker::edgar::{edgar_container_names, format_remaining_edgars_string, EDGAR_LEADER_NAME};
use crate::core::{TheoError, SLEEP_TIME_SECONDS, TIMEOUT_SECONDS};

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

pub fn wait_until_all_edgar_peers_are(target_status: EdgarDeploymentStatus) -> crate::Result {
    println!("STAGE: EDGAR {target_status}");

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
                EdgarDeploymentStatus::Provisioned => {
                    if edgar::check_edgar_container_provisioning_done(&edgar_name)? {
                        println!("EDGAR peer '{edgar_name}' is provisioned.");
                        remaining_edgar_names.remove(&edgar_name);
                    }
                }
                EdgarDeploymentStatus::Ready => {
                    println!("Checking if EDGAR peer '{edgar_name}' is ready...");
                    let leader_arg = if edgar_name == EDGAR_LEADER_NAME { "leader" } else { "peer" };
                    DockerCommand::new_exec(&edgar_name)
                        .arg("/opt/wait_until_ready.sh")
                        .arg(leader_arg)
                        .expect_status(&format!("Failed to check if EDGAR peer '{edgar_name}' is ready."))?;
                    println!("EDGAR peer '{edgar_name}' is ready.");
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
