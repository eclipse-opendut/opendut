use anyhow::{anyhow, Error};

use crate::core::docker::command::DockerCommand;
use crate::core::TheoError;

pub const EDGAR_LEADER_NAME: &str = "edgar-leader";
pub const EDGAR_PEER_1_NAME: &str = "edgar-peer-1";
pub const EDGAR_PEER_2_NAME: &str = "edgar-peer-2";

pub(crate) fn edgar_container_names() -> Result<Vec<String>, Error> {
    let edgar_names = DockerCommand::new()
        .arg("ps")
        .arg("--all")
        .arg(r#"--format="{{.Names}}""#)
        .arg("--filter")
        .arg("name=edgar-*").output();
    match edgar_names {
        Ok(output) => {
            let stdout = String::from_utf8(output.stdout)?;
            let result = stdout.lines()
                .map(|s| s.trim_matches('"').to_string()).collect();
            Ok(result)
        }
        Err(error) => {
            Err(anyhow!(TheoError::DockerCommandFailed(format!("Failed to get edgar container names. Cause: {}", error))))
        }
    }
}
