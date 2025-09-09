use std::net::Ipv4Addr;
use std::str::FromStr;
use std::sync::Arc;

use anyhow::{Context, Result};
use async_trait::async_trait;
use tracing::{debug, warn};

use opendut_netbird_client_api::extension::LocalPeerStateExtension;
use opendut_model::util::net::NetworkInterfaceName;

use crate::service::network_interface::gre;
use crate::service::network_interface::manager::NetworkInterfaceManagerRef;
use crate::setup::Leader;
use crate::common::task::{Success, Task, TaskStateFulfilled};

pub struct CreateGreInterfaces {
    pub network_interface_manager: NetworkInterfaceManagerRef,
    pub leader: Leader,
    pub bridge_name: NetworkInterfaceName,
}

#[async_trait]
impl Task for CreateGreInterfaces {
    fn description(&self) -> String {
        String::from("Create GRE interfaces for Peers")
    }
    async fn check_present(&self) -> Result<TaskStateFulfilled> {
        Ok(TaskStateFulfilled::Unchecked)
    }
    async fn make_present(&self) -> Result<Success> {
        let mut netbird_client = opendut_netbird_client_api::client::Client::connect().await?;

        let full_status = netbird_client.full_status().await
            .context("Error during NetBird-Status")?;

        let local_ip = full_status.local_peer_state.unwrap().local_ip()?;

        let leader = {
            let mut leader = self.leader.clone();
            if let Leader::Remote(remote_ip) = leader {
                if remote_ip == local_ip {
                    debug!("Address of Leader::Remote '{remote_ip}' is local address. Continuing as Leader::Local.");
                    leader = Leader::Local;
                }
            }
            leader
        };

        if let Leader::Remote(remote_ip) = leader {
            //Create GRE interface to leader.
            gre::setup_interfaces(&local_ip, &[remote_ip], &self.bridge_name, Arc::clone(&self.network_interface_manager)).await?;

            Ok(Success::message(String::from("Interface to leader created")))
        }
        else {
            //Create GRE interfaces for all peers.
            let remote_ips = full_status.peers.into_iter()
                .filter_map(|peer| {
                    let remote_ip = peer.ip;
                    let address = Ipv4Addr::from_str(&remote_ip)
                        .context(format!("Failed to parse remote IP returned by NetBird '{remote_ip}'."));
                    match address {
                        Ok(address) => Some(address),
                        Err(cause) => {
                            warn!("Discarding IP address returned by NetBird: {cause}");
                            None
                        }
                    }
                })
                .collect::<Vec<_>>();

            let number_of_remote_ips = remote_ips.len();

            gre::setup_interfaces(&local_ip, &remote_ips, &self.bridge_name, Arc::clone(&self.network_interface_manager)).await?;

            Ok(Success::message(format!("{number_of_remote_ips} interface(s) created; acting as leader with IP address '{local_ip}'")))
        }
    }
}
