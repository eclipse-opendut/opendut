use std::process::Stdio;
use std::time::Duration;
use backoff::ExponentialBackoffBuilder;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tracing::{error, trace};
use crate::service::network_metrics::rperf::{RperfError, RperfRunError};
use crate::service::network_metrics::rperf::RperfRunError::RperfServerError;

pub async fn exponential_backoff_launch_rperf_server(
    rperf_backoff_max_elapsed_time_ms: Duration,
) -> Result<(), RperfRunError> {
    let exponential_backoff = ExponentialBackoffBuilder::default()
        .with_max_elapsed_time(Some(rperf_backoff_max_elapsed_time_ms))
        .build();

    let backoff_result = backoff::future::retry(
        exponential_backoff,
        || async {
            launch_rperf_server().await?;
            Ok(())
        }
    ).await;

    backoff_result
        .map_err(|cause| RperfServerError { message: "Could not run rperf server".to_string(), cause })
}

pub async fn launch_rperf_server() -> Result<(), RperfError> {

    let rperf_server = Command::new(crate::common::constants::rperf::executable_install_file())
        .arg("--server")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();

    match rperf_server {
        Ok(mut rperf_server) => {
            let stdout = rperf_server.stdout.take().expect("rperf server stdout could not be read");
            let stderr = rperf_server.stderr.take().expect("rperf server stderr could not be read");

            let stdout_lines = BufReader::new(stdout).lines();
            let stderr_lines = BufReader::new(stderr).lines();

            tokio::spawn(async move {
                let mut stdout_lines = stdout_lines;
                while let Some(line) = stdout_lines.next_line().await.expect("Failed to read next line in rperf server stdout") {
                    trace!("Rperf Server STDOUT: {}", line);
                }
            });

            tokio::spawn(async move {
                let mut stderr_lines = stderr_lines;
                while let Some(line) = stderr_lines.next_line().await.expect("Failed to read next line in rperf server stderr") {
                    if line.contains(" ERROR rperf::server") {
                        error!("Rperf Server STDERR: {}", line);
                    } else {
                        trace!("Rperf Server STDERR: {}", line);
                    }
                }
            });
            Ok(())
        }
        Err(error) => {
            error!("The rperf server could not be started: {}", error);
            Err(RperfError::Start { message: "The rperf server could not be started".to_string(), cause: error })
        }
    }
}
