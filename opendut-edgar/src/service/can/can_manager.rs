use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use anyhow::bail;
use opendut_model::cluster::PeerClusterAssignment;
use opendut_model::util::Port;

use tracing::{debug, error, info};
use opendut_model::peer::PeerId;
use opendut_model::util::net::{NetworkInterfaceDescriptor, NetworkInterfaceName};

use crate::service::can::cannelloni_manager::CannelloniManager;
use crate::service::network_interface::manager::NetworkInterfaceManagerRef;

pub type CanManagerRef = Arc<CanManager>;

pub struct CanManager {
    /*
        The cannelloni_termination_token is used to signal the CannelloniManagers, running in separate threads, to terminate. Once it is read as 'true' 
        by a CannelloniManager, it will terminate. 
        The idea is that, with every Cluster Assignment pushed from CARL, a new AtomicBool ('false') is created that is shared between all newly started 
        CannelloniManagers and the CanManager. 
        Once a new Cluster Assignment is pushed, the CanManager sets the AtomicBool to 'true' and forgets it by replacing the content of 
        cannelloni_termination_token with a new AtomicBool, to be used for the new generation of CannelloniManagers.
        The old generation of CannelloniManagers can now read the old AtomicBool and terminate accordingly.
     */
    cannelloni_termination_token: Mutex<Arc<AtomicBool>>,
    network_interface_manager: NetworkInterfaceManagerRef,
}

impl CanManager {
    pub fn create(network_interface_manager: NetworkInterfaceManagerRef) -> CanManagerRef {
        Arc::new(Self {
            cannelloni_termination_token: Mutex::new(Arc::new(AtomicBool::new(false))),
            network_interface_manager
        })
    }

    pub async fn setup_local_routing(
        &self,
        bridge_name: &NetworkInterfaceName,
        local_can_interfaces: Vec<NetworkInterfaceDescriptor>,
    ) -> Result<(), Error> {

        self.create_can_bridge(bridge_name).await
            .map_err(|cause| Error::Other { message: format!("Error while creating CAN bridge: {cause}") })?;


        Ok(())
    }
    
    async fn create_can_bridge(&self, bridge_name: &NetworkInterfaceName) -> anyhow::Result<()> {

        if self.network_interface_manager.find_interface(bridge_name).await?.is_none() {
            debug!("Creating CAN bridge '{bridge_name}'.");
            let bridge = self.network_interface_manager.create_vcan_interface(bridge_name).await?;
            self.network_interface_manager.set_interface_up(&bridge).await?;
        } else {
            debug!("Not creating CAN bridge '{bridge_name}', because it already exists.");
        }

        Ok(())
    }

    async fn update_can_interface(&self, interface: &NetworkInterfaceDescriptor) -> anyhow::Result<()> {
        if let Some(network_interface) = self.network_interface_manager.find_interface(&interface.name).await? {
            self.network_interface_manager.set_interface_down(&network_interface).await?;
            if let Err(cause) = self.network_interface_manager.update_interface(interface.to_owned()).await {
                error!("Error updating CAN interface - A possible reason might be, that a virtual CAN interface was used: {cause}");
            };
            self.network_interface_manager.set_interface_up(&network_interface).await?;

        } else {
            bail!("Cannot find CAN interface with name: '{}'.", interface.name);
        }
        
        Ok(())
    }

    async fn terminate_cannelloni_managers(&self) {
        self.cannelloni_termination_token.lock().unwrap().store(true, Ordering::Relaxed);
    }
    
    pub async fn setup_remote_routing_client(&self, bridge_name: &NetworkInterfaceName, leader_ip: &IpAddr, leader_port: &Port) -> Result<(), Error> {

        self.terminate_cannelloni_managers().await;

        let mut guarded_termination_token = self.cannelloni_termination_token.lock().unwrap();
        *guarded_termination_token = Arc::new(AtomicBool::new(false));

        info!("Spawning cannelloni manager as client");

        // TODO: The buffer timeout here should likely be configurable through CARL (cannot be 0)
        let mut cannelloni_manager = CannelloniManager::new (
            false,
            bridge_name.clone(),
            *leader_port,
            *leader_ip,
            Duration::from_micros(1),
            guarded_termination_token.clone(),
        );

        tokio::spawn(async move {
            cannelloni_manager.run().await;
        });

        Ok(())
    }

    pub async fn setup_remote_routing_server(
        &self,
        bridge_name: &NetworkInterfaceName,
        remote_assignments: HashMap<PeerId, PeerClusterAssignment>,
    ) -> Result<(), Error>  {

        self.terminate_cannelloni_managers().await;

        let mut guarded_termination_token = self.cannelloni_termination_token.lock().unwrap();
        *guarded_termination_token = Arc::new(AtomicBool::new(false));


        for (_, remote_assignment) in remote_assignments {
            info!("Spawning cannelloni manager as server for peer with IP {}", remote_assignment.vpn_address);
    
            let mut cannelloni_manager = CannelloniManager::new(
                true,
                bridge_name.clone(),
                remote_assignment.can_server_port,
                remote_assignment.vpn_address,
                Duration::from_micros(1),
                guarded_termination_token.clone()
            );

            tokio::spawn(async move {
                cannelloni_manager.run().await;
            });
        }

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
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
