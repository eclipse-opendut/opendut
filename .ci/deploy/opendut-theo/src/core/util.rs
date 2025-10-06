use std::io;
use std::path::Path;
use std::process::Output;
use anyhow::Error;
use crate::core::TheoError;


pub(crate) fn consume_output(output: io::Result<Output>) -> Result<String, TheoError> {
    let output = output.map_err(|error| TheoError::ConsumeOutputError(format!("Failed to consume output: {error:?}")))?;

    if output.status.code().unwrap_or(1) != 0 {
        Err(TheoError::ConsumeOutputError(format!("Failed to execute command: {output:?}")))
    } else {
        Ok(output.stdout
            .iter()
            .map(|&c| c as char)
            .collect::<String>()
            .trim()
            .to_string())
    }
}

pub fn file_modified_time_in_seconds<P: AsRef<Path>>(path: &P) -> Result<u64, Error> {
    Ok(std::fs::metadata(path)?
        .modified()?
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs())
}