use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;
use backoff::ExponentialBackoffBuilder;
use opentelemetry::{global, KeyValue};
use opentelemetry::metrics::Gauge;
use regex::Regex;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::Mutex;
use tracing::{debug, error, trace};
use opendut_types::cluster::PeerClusterAssignment;
use crate::service::network_metrics::rperf::{RperfError, RperfRunError};
use crate::service::network_metrics::rperf::RperfRunError::RperfClientError;

pub async fn launch_rperf_clients(peers: Vec<PeerClusterAssignment>, target_bandwidth_kbit_per_second: u64, rperf_backoff_max_elapsed_time_ms: Duration) {

    let meter = global::meter(opendut_util::logging::DEFAULT_METER_NAME);

    let megabits_second_send = meter.f64_gauge("megabits_second_send").init();
    let megabits_second_receive = meter.f64_gauge("megabits_second_receive").init();

    let megabits_second_send_mutex = Arc::new(Mutex::new(megabits_second_send));
    let megabits_second_receive_mutex = Arc::new(Mutex::new(megabits_second_receive));

    for peer in peers.clone() {
        let megabits_second_send_mutex = megabits_second_send_mutex.clone();
        let megabits_second_receive_mutex = megabits_second_receive_mutex.clone();
        tokio::spawn(async move {
            exponential_backoff_launch_rperf_client(
                &peer,
                target_bandwidth_kbit_per_second,
                rperf_backoff_max_elapsed_time_ms,
                megabits_second_send_mutex,
                megabits_second_receive_mutex
            ).await
                .inspect_err(|cause| error!("Failed to start rperf client for peer {peer_id}: {cause}", peer_id=peer.peer_id))
        });
    }
}

pub async fn exponential_backoff_launch_rperf_client(
    peer: &PeerClusterAssignment,
    target_bandwidth_kbit_per_second: u64,
    rperf_backoff_max_elapsed_time_ms: Duration,
    megabits_second_send_mutex: Arc<Mutex<Gauge<f64>>>,
    megabits_second_receive_mutex: Arc<Mutex<Gauge<f64>>>
) -> Result<(), RperfRunError> {
    let exponential_backoff = ExponentialBackoffBuilder::default()
        .with_max_elapsed_time(Some(rperf_backoff_max_elapsed_time_ms))
        .build();

    let backoff_result = backoff::future::retry(
        exponential_backoff,
        || async {
            launch_rperf_client(peer, target_bandwidth_kbit_per_second, &megabits_second_send_mutex, &megabits_second_receive_mutex).await?;
            Ok(())
        }
    ).await;

    backoff_result
        .map_err(|cause| RperfClientError { message: "Could not run rperf client".to_string(), cause })
}

pub async fn launch_rperf_client(
    peer: &PeerClusterAssignment,
    target_bandwidth_kbit_per_second: u64,
    megabits_second_send_mutex: &Arc<Mutex<Gauge<f64>>>,
    megabits_second_receive_mutex: &Arc<Mutex<Gauge<f64>>>
) -> Result<(), RperfError> {
    let rperf_client = Command::new(crate::common::constants::rperf::executable_install_file())
        .arg("--client")
        .arg(peer.vpn_address.to_string())
        .arg("--bandwidth")
        .arg(format!("{target_bandwidth_kbit_per_second}k")) //the k suffix signifies the entered bandwidth is to be read in kilobits
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();

    match rperf_client {
        Ok(mut rperf_client) => {
            match rperf_client.stdout.take() {
                Some(stdout) => {
                    let stdout_lines = BufReader::new(stdout).lines();
                    let megabits_pattern = Regex::new(r"megabits/second: (\d+\.\d+)").expect("Failed to compile rperf regex");
                    let mut operation: RperfOperation = RperfOperation::Default;
                    let mut stdout_lines = stdout_lines;

                    while let Ok(Some(line)) = stdout_lines.next_line().await {
                        if line.contains("stream-average bytes per second") {
                            trace!("Rperf Client STDOUT: {}", line);
                            if let Some(captures) = megabits_pattern.captures(&line) {
                                if let Some(number) = captures.get(1) {
                                    let number_str = number.as_str();
                                    match operation {
                                        RperfOperation::Send => match number_str.parse::<f64>() {
                                            Ok(value) => {
                                                megabits_second_send_mutex.lock().await
                                                    .record(value, &[KeyValue::new("peer_ip_address", peer.vpn_address.to_string())]);
                                                debug!("Sending to {} in megabits/second: {}", peer.vpn_address.to_string(), value);
                                            },
                                            Err(cause) => {
                                                error!("Failed to parse rperf bandwidth: {}", cause);
                                                return Err(RperfError::BandwidthParse { message: "Failed to parse rperf bandwidth".to_string(), cause });
                                            }
                                        },
                                        RperfOperation::Receive => match number_str.parse::<f64>() {
                                            Ok(value) => {
                                                megabits_second_receive_mutex.lock().await
                                                    .record(value, &[KeyValue::new("peer_ip_address", peer.vpn_address.to_string())]);
                                                debug!("Receiving from {} in megabits/second: {}", peer.vpn_address.to_string(), value);
                                            },
                                            Err(cause) => {
                                                error!("Failed to parse rperf bandwidth: {}", cause);
                                                return Err(RperfError::BandwidthParse { message: "Failed to parse rperf bandwidth".to_string(), cause });
                                            }
                                        },
                                        RperfOperation::Default => {} //do nothing
                                    }
                                }
                            }
                        }
                        if line.contains("TCP send result over ") {
                            operation = RperfOperation::Send;
                        } else if line.contains("TCP receive result over ") {
                            operation = RperfOperation::Receive;
                        } else {
                            operation = RperfOperation::Default;
                        }
                    }
                }
                None => {
                    error!("Failed to access rperf client stdout");
                    return Err(RperfError::StdoutAccess { message: "Failed to access rperf client stdout".to_string() });
                }
            }

            match rperf_client.stderr.take() {
                Some(stderr) => {
                    let stderr_lines = BufReader::new(stderr).lines();
                    let mut stderr_lines = stderr_lines;

                    while let Ok(Some(line)) = stderr_lines.next_line().await {
                        if line.contains(" ERROR rperf") {
                            error!("Rperf Client STDERR: {}", line);
                            return Err(RperfError::Other { message: format!("rperf client STDERR: {}", line) })
                        } else {
                            trace!("Rperf Client STDERR: {}", line);
                        }
                    }
                }
                None => {
                    error!("Failed to access rperf client stderr");
                    return Err(RperfError::StderrAccess { message: "Failed to access rperf client stderr".to_string() });
                }
            }
            Ok(())
        }
        Err(error) => {
            error!("The rperf client could not be started: {}", error);
            Err(RperfError::Start { message: "The rperf client could not be started".to_string(), cause: error })
        }
    }
}

#[derive (PartialEq)]
enum RperfOperation {
    Send,
    Receive,
    Default
}
