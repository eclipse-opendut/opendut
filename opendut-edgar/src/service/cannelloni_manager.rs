use tokio::process::{Child, Command};
use tokio::io::{AsyncReadExt, BufReader};
use std::process::Stdio;
use std::net::IpAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info};
use opendut_types::util::net::NetworkInterfaceName;
use opendut_types::util::Port;

pub struct CannelloniManager{
    is_server: bool, 
    can_if_name: NetworkInterfaceName, 
    server_port: Port, 
    remote_ip: IpAddr, 
    buffer_timeout: Duration,
    termination_request_token: Arc<AtomicBool>,
    cannelloni_proc: Option<Child>,
}

enum MonitorResult {
    RestartCannelloni,
    TerminateManager,
}

const MONITOR_INTERVAL_MS: u64 = 100;

// TODO: Implement exponential back-off when restarting cannelloni?
impl CannelloniManager {

    pub fn new(is_server: bool, can_if_name: NetworkInterfaceName, server_port: Port, remote_ip: IpAddr, buffer_timeout: Duration, termination_request_token: Arc<AtomicBool>) -> Self {
        Self { 
            is_server, 
            can_if_name, 
            server_port, 
            remote_ip,
            buffer_timeout, 
            termination_request_token,
            cannelloni_proc: None
        }
    }
    
    pub async fn run(&mut self) {
        loop {
            let mut cmd = Command::new("cannelloni");
            self.fill_cannelloni_cmd(&mut cmd).await;

            match cmd.spawn() {
                Ok(child) => {
                    info!("Spawned cannelloni thread for remote IP {}.", self.remote_ip.to_string());
                    self.cannelloni_proc = Some(child);

                    match self.monitor_process().await {
                        MonitorResult::RestartCannelloni => (),
                        MonitorResult::TerminateManager => {
                            self.kill_cannelloni_process().await;
                            return
                        },
                    }
                },
                Err(err) => error!("Failed to start cannelloni instance for remote IP {}: '{}'.", self.remote_ip.to_string(), err),
            }
        }
    }

    async fn kill_cannelloni_process(&mut self) {
        match self.cannelloni_proc.as_mut().unwrap().kill().await {
            Ok(_) => (),
            Err(err) => error!("Failed to start cannelloni instance for remote IP {}: '{}'.", self.remote_ip.to_string(), err),
        }
    }

    async fn monitor_process(&mut self) -> MonitorResult {
        loop {
            match self.cannelloni_proc.as_mut().unwrap().try_wait() {
                Ok(op) => {
                    if op.is_some() {
                        self.handle_premature_termination().await;
                        return MonitorResult::RestartCannelloni
                    }
                },
                Err(err) => error!("Failed to get status of cannelloni instance for remote IP {}: '{}'.", self.remote_ip.to_string(), err)
            }

            if self.termination_request_token.load(Ordering::Relaxed) {
                return MonitorResult::TerminateManager
            }

            tokio::time::sleep(std::time::Duration::from_millis(MONITOR_INTERVAL_MS)).await;
        }
    }


    async fn handle_premature_termination(&mut self) {

        let stdout = match self.cannelloni_proc.as_mut().unwrap().stdout.take() {
            Some(stdout) => stdout,
            None => {
                error!("Cannelloni for remote IP {} terminated prematurely but failed to get stdout.", self.remote_ip.to_string());
                return;
            }
        };
        let mut stdout_reader = BufReader::new(stdout);
        let mut stdout_u8: Vec<u8> = Vec::new();
        let _ = stdout_reader.read_to_end(&mut stdout_u8).await;
        let stdout_str = String::from_utf8_lossy(&stdout_u8);

        let stderr = match self.cannelloni_proc.as_mut().unwrap().stderr.take() {
            Some(stderr) => stderr,
            None => {
                error!("Cannelloni for remote IP {} terminated prematurely but failed to get stderr.", self.remote_ip.to_string());
                return;
            }
        };
        let mut stderr_reader = BufReader::new(stderr);
        let mut stderr_u8: Vec<u8> = Vec::new();
        let _ = stderr_reader.read_to_end(&mut stderr_u8).await;
        let stderr_str = String::from_utf8_lossy(&stderr_u8);

        error!(
            "Cannelloni for remote IP {} terminated prematurely with stderr:\n{}\nstdout:\n{}", 
            self.remote_ip.to_string(), 
            stdout_str,
            stderr_str
        )
    }


    async fn fill_cannelloni_cmd(&self, cmd: &mut Command) {
        let instance_type = if self.is_server {"s"} else {"c"};
        let port_arg = if self.is_server {"-l"} else {"-r"};

        cmd.arg("-I")
            .arg(self.can_if_name.name())
            .arg("-S")
            .arg(instance_type)
            .arg("-t")
            .arg(self.buffer_timeout.as_micros().to_string())
            .arg("-R")
            .arg(self.remote_ip.to_string())
            .arg(port_arg)
            .arg(self.server_port.to_string())
            .stderr(Stdio::piped())
            .stdout(Stdio::piped());

    }
    
}
