use anyhow::{anyhow, Error};
use std::collections::HashSet;

use crate::core::docker::command::DockerCommand;
use crate::core::TheoError;

pub const EDGAR_LEADER_NAME: &str = "edgar-leader";
pub const EDGAR_PEER_1_NAME: &str = "edgar-peer-1";
pub const EDGAR_PEER_2_NAME: &str = "edgar-peer-2";

pub(crate) fn edgar_container_names() -> Result<HashSet<String>, Error> {
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
                .map(|s| s.trim_matches('"').to_string())
                .collect();
            Ok(result)
        }
        Err(error) => {
            Err(anyhow!(TheoError::DockerCommandFailed(format!("Failed to get edgar container names. Cause: {error}"))))
        }
    }
}

pub(crate) fn format_remaining_edgars_string(remaining_edgar_names: &HashSet<String>) -> String {
    let mut remaining_edgar_names = remaining_edgar_names.iter()
        .cloned()
        .collect::<Vec<_>>();

    remaining_edgar_names.sort();

    remaining_edgar_names.join(", ")
}
