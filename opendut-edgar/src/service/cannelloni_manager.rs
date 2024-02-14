use tokio::process::Command;
use std::process::Stdio;
use std::{net::IpAddr, process::Output};
use opendut_types::util::net::NetworkInterfaceName;

pub struct CannelloniManager{
    pub is_server: bool, 
    pub can_if_name: NetworkInterfaceName, 
    pub server_port: u16, 
    pub remote_ip: IpAddr, 
    pub buffer_timeout: u64,
}

// TODO: Allow terminating cannelloni to set up everything new when new configs are pushed from CARL
// TODO: Implement exponential back-off when restarting cannelloni?
impl CannelloniManager {
    pub async fn run(&mut self) {
        loop {
            let mut cmd = Command::new("cannelloni");
            self.fill_cannelloni_cmd(&mut cmd).await;

            match cmd.spawn() {
                Ok(child) => {
                    log::info!("Spawned cannelloni thread for remote IP {}.", self.remote_ip.to_string());
                    let out = child.wait_with_output().await;
                    self.handle_cannelloni_termination(out);
                },
                Err(err) => log::error!("Failed to start cannelloni instance for remote IP {}: '{}'.", self.remote_ip.to_string(), err),
            }
        }
    }

    fn handle_cannelloni_termination(&mut self, cannelloni_res: Result<Output, std::io::Error>) {
        match cannelloni_res {
            Ok(out) => {
                log::error!(
                    "Cannelloni for remote IP {} terminated prematurely with stderr:\n{}\nstdout:\n{}", 
                    self.remote_ip.to_string(), 
                    String::from_utf8_lossy(&out.stderr),
                    String::from_utf8_lossy(&out.stdout)
                )
            },
            Err(err) => {
                log::error!("Cannelloni for remote IP {} terminated prematurely but failed to get output: '{}'.", self.remote_ip.to_string(), err)
            }
        }
    }


    async fn fill_cannelloni_cmd(&mut self, cmd: &mut Command) {
        let instance_type = if self.is_server {"s"} else {"c"};
        let port_arg = if self.is_server {"-l"} else {"-r"};

        cmd.arg("-I")
            .arg(self.can_if_name.name())
            .arg("-S")
            .arg(instance_type)
            .arg("-t")
            .arg(self.buffer_timeout.to_string())
            .arg("-R")
            .arg(self.remote_ip.to_string())
            .arg(port_arg)
            .arg(self.server_port.to_string())
            .stderr(Stdio::piped())
            .stdout(Stdio::piped());

    }
    
}
