use std::process::Stdio;
use std::time::Duration;
use backon::Retryable;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tracing::{error, trace};
use crate::service::network_metrics::manager::Spawner;
use crate::service::network_metrics::rperf::{RperfError, RperfRunError};
use crate::service::network_metrics::rperf::RperfRunError::RperfServerError;

pub async fn exponential_backoff_launch_rperf_server(
    spawner: Spawner,
    rperf_backoff_max_elapsed_time_ms: Duration,
) -> Result<(), RperfRunError> {
    let exponential_backoff = backon::ExponentialBuilder::default()
        .with_max_delay(rperf_backoff_max_elapsed_time_ms);

    let backoff_result = (|| async {
            launch_rperf_server(spawner.clone()).await?;
            Ok(())
        })
        .retry(exponential_backoff)
        .await;

    backoff_result
        .map_err(|cause| RperfServerError { message: "Could not run rperf server".to_string(), cause })
}

pub async fn launch_rperf_server(spawner: Spawner) -> Result<(), RperfError> {

    let rperf_server = Command::new(crate::common::constants::rperf::executable_install_file())
        .arg("--server")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn();

    match rperf_server {
        Ok(mut rperf_server) => {
            let stdout = rperf_server.stdout.take().expect("rperf server stdout could not be read");
            let stderr = rperf_server.stderr.take().expect("rperf server stderr could not be read");

            let mut stdout_lines = BufReader::new(stdout).lines();
            let mut stderr_lines = BufReader::new(stderr).lines();

            spawner.lock().await
                .spawn(async move {
                    while let Some(line) = stdout_lines.next_line().await.expect("Failed to read next line in rperf server stdout") {
                        trace!("Rperf Server STDOUT: {}", line);
                    }
                });

            spawner.lock().await
                .spawn(async move {
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
