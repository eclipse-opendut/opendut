use std::io;
use std::process::Output;
use crate::core::TheoError;


pub(crate) fn consume_output(output: io::Result<Output>) -> Result<String, TheoError> {
    let output = output.map_err(|e| TheoError::ConsumeOutputError(format!("Failed to consume output: {:?}", e)))?;

    if output.status.code().unwrap_or(1) != 0 {
        Err(TheoError::ConsumeOutputError(format!("Failed to execute command: {:?}", output)))
    } else {
        Ok(output.stdout
            .iter()
            .map(|&c| c as char)
            .collect::<String>()
            .trim()
            .to_string())
    }
}
