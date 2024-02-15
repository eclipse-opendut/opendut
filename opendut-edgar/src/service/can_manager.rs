use std::net::IpAddr;
use std::sync::{Arc, Mutex};
use regex::Regex;

use tokio::process::Command;
use tokio_util::sync::CancellationToken;

use opendut_types::util::net::NetworkInterfaceName;

use crate::service::cannelloni_manager::CannelloniManager;
use crate::service::network_interface;
use crate::service::network_interface::manager::NetworkInterfaceManagerRef;

pub type CanManagerRef = Arc<CanManager>;

pub struct CanManager{
    cannelloni_cancellation_token: Mutex<Option<CancellationToken>>,
    network_interface_manager: NetworkInterfaceManagerRef,
}

impl CanManager {
    pub fn create(network_interface_manager: NetworkInterfaceManagerRef) -> Self {

        Self { cannelloni_cancellation_token: Mutex::new(None) , network_interface_manager}
    }

    async fn check_can_route_exists(&self, src: &NetworkInterfaceName, dst: &NetworkInterfaceName, can_fd: bool, max_hops: u8) -> Result<bool, Error> {
        let output = Command::new("cangw")
                .arg("-L")
                .output()
                .await
                .map_err(|cause| Error::CommandLineProgramExecution { command: "cangw".to_string(), cause })?;
        
        // cangw -L returns non-zero exit code despite succeeding, so we don't check it here
        
        let output_str = String::from_utf8_lossy(&output.stdout);

        let re = Regex::new(r"(?m)^cangw -A -s ([^\n ]+) -d ([^\n ]+) ((?:-X )?)-e -l ([0-9[^\n ]]+) #.*$").unwrap();

        for (_, [exist_src, exist_dst, can_fd_flag, exist_max_hops]) in re.captures_iter(&output_str).map(|c| c.extract()) {
            let exist_can_fd = can_fd_flag.trim() == "-X";
            if exist_src == src.to_string() && exist_dst == dst.to_string() && exist_can_fd == can_fd && exist_max_hops == max_hops.to_string(){
                return Ok(true)
            }

        }

        Ok(false)
    }

    async fn create_can_route(&self, src: &NetworkInterfaceName, dst: &NetworkInterfaceName, can_fd: bool, max_hops: u8) -> Result<(), Error> {
        self.network_interface_manager.try_find_interface(&src).await?;
        self.network_interface_manager.try_find_interface(&dst).await?;

        let mut cmd = Command::new("cangw");
        cmd.arg("-A")
            .arg("-s")
            .arg(src.name())
            .arg("-d")
            .arg(dst.name())
            .arg("-e")
            .arg("-l")
            .arg(max_hops.to_string());

        if can_fd {
            cmd.arg("-X");
        } 

        let output= cmd.output().await
                .map_err(|cause| Error::CommandLineProgramExecution { command: "cangw".to_string(), cause })?;

        if ! output.status.success() {
            return Err(Error::CanRouteCreation { 
                src: src.clone(), 
                dst: dst.clone(), 
                cause: format!("{:?}", String::from_utf8_lossy(&output.stderr).trim()) });
        }

        self.check_can_route_exists(src, dst, can_fd, max_hops).await?.then(|| ()).ok_or(Error::CanRouteCreationNoCause { src: src.clone(), dst: dst.clone() })?;

        Ok(())
    }

    async fn remove_all_can_routes(&self) -> Result<(), Error> {
        let output = Command::new("cangw")
                    .arg("-F")
                    .output()
                    .await
                    .map_err(|cause| Error::CommandLineProgramExecution { command: "cangw".to_string(), cause })?;

        if ! output.status.success() {
            return Err(Error::CanRouteFlushing { cause: format!("{:?}", String::from_utf8_lossy(&output.stderr).trim()) });
        }
        Ok(())
    }

    pub async fn setup_local_routing(
        &self,
        bridge_name: &NetworkInterfaceName,
        local_can_interfaces: Vec<NetworkInterfaceName>,
    ) -> Result<(), Error> {
    
    
        self.create_can_bridge(bridge_name).await
            .map_err(|cause| Error::Other { message: format!("Error while creating CAN bridge: {cause}") })?;
    
        self.remove_all_can_routes().await?;
    
        for interface in local_can_interfaces {
            self.create_can_route(bridge_name, &interface, true, 2).await?;
            self.create_can_route(bridge_name, &interface, false, 2).await?;
            self.create_can_route(&interface, bridge_name, true, 2).await?;
            self.create_can_route(&interface, bridge_name, false, 2).await?;
        }
    
        Ok(())
    }
    
    async fn create_can_bridge(&self, bridge_name: &NetworkInterfaceName) -> anyhow::Result<()> {
    
        if self.network_interface_manager.find_interface(bridge_name).await?.is_none() {
            log::debug!("Creating CAN bridge '{bridge_name}'.");
            let bridge = self.network_interface_manager.create_vcan_interface(bridge_name).await?;
            self.network_interface_manager.set_interface_up(&bridge).await?;
        } else {
            log::debug!("Not creating CAN bridge '{bridge_name}', because it already exists.");
        }
    
        Ok(())
    }
    
    // TODO: determining the port for cannelloni like this is a bit dirty, we should get that information from CARL instead
    // Takes the last two bytes of the IP address to be used as the port
    fn peer_ip_to_leader_port(&self, peer_ip: &IpAddr) -> anyhow::Result<u16>{
        assert!(peer_ip.is_ipv4());
        let ip_bytes: Vec<u8> = peer_ip.to_string().split(".").map(|b| b.parse::<u8>().unwrap()).collect();
        let port = ((ip_bytes[2] as u16) << 8) | ip_bytes[3] as u16;
        Ok(port)
    }

    async fn terminate_cannelloni_managers(&self) {
        let mut guarded_cancellation_token = self.cannelloni_cancellation_token.lock().unwrap();
        match &mut *guarded_cancellation_token {
            Some(cancellation_token) => {
                cancellation_token.cancel();
                *guarded_cancellation_token = None;
            },
            None => (),
        }
    }
    
    pub async fn setup_remote_routing_client(&self, bridge_name: &NetworkInterfaceName, local_ip: &IpAddr, leader_ip: &IpAddr) -> Result<(), Error> {

        self.terminate_cannelloni_managers().await;
    
        let leader_port = self.peer_ip_to_leader_port(local_ip).unwrap();

        let token = CancellationToken::new();
        let cloned_token = token.clone();
        let mut guarded_cancellation_token = self.cannelloni_cancellation_token.lock().unwrap();
        *guarded_cancellation_token = Some(token);
        
        log::info!("Spawning cannelloni manager as client");
    
        // TODO: The buffer timeout here should likely be configurable through CARL (cannot be 0)
        let mut cannelloni_manager = CannelloniManager::new (
            false, 
            bridge_name.clone(), 
            leader_port, 
            leader_ip.clone(), 
            1,
            cloned_token,
        );
    
        tokio::spawn(async move {
            cannelloni_manager.run().await;
        });
    
        Ok(())
    }
    
    pub async fn setup_remote_routing_server(&self, bridge_name: &NetworkInterfaceName, remote_ips: &Vec<IpAddr>) -> Result<(), Error>  {

        self.terminate_cannelloni_managers().await;

        let cancellation_token = CancellationToken::new();
        
    
        for remote_ip in remote_ips {
            let leader_port = self.peer_ip_to_leader_port(&remote_ip).unwrap();

            let cloned_cancellation_token = cancellation_token.clone();

            log::info!("Spawning cannelloni manager as server for peer with IP {}", remote_ip.to_string());
    
            let mut cannelloni_manager = CannelloniManager::new(
                true, 
                bridge_name.clone(), 
                leader_port, 
                remote_ip.clone(), 
                1,
                cloned_cancellation_token
            );
        
            tokio::spawn(async move {
                cannelloni_manager.run().await;
            });
        }

        let mut guarded_cancellation_token = self.cannelloni_cancellation_token.lock().unwrap();
        *guarded_cancellation_token = Some(cancellation_token);    
    
        Ok(())
    }

}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Error while managing CAN interfaces: {0}")]
    NetworkInterfaceError(#[from] network_interface::manager::Error),
    #[error("Failure while invoking command line program '{command}': {cause}")]
    CommandLineProgramExecution { command: String, cause: std::io::Error },
    #[error("Failure while creating CAN route '{src}' -> '{dst}': {cause}")]
    CanRouteCreation { src: NetworkInterfaceName, dst: NetworkInterfaceName, cause: String },
    #[error("Failure while creating CAN route '{src}' -> '{dst}'")]
    CanRouteCreationNoCause { src: NetworkInterfaceName, dst: NetworkInterfaceName},
    #[error("Failure while flushing existing CAN routes: {cause}")]
    CanRouteFlushing { cause: String },
    #[error("{message}")]
    Other { message: String },
}