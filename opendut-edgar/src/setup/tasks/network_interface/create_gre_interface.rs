use std::net::Ipv4Addr;
use std::str::FromStr;
use std::sync::Arc;

use anyhow::{Context, Result};
use futures::executor::block_on;

use opendut_netbird_client_api::extension::LocalPeerStateExtension;
use opendut_types::util::net::NetworkInterfaceName;

use crate::service::network_interface::gre;
use crate::service::network_interface::manager::NetworkInterfaceManagerRef;
use crate::setup::Router;
use crate::setup::task::{Success, Task, TaskFulfilled};

pub struct CreateGreInterfaces {
    pub network_interface_manager: NetworkInterfaceManagerRef,
    pub router: Router,
    pub bridge_name: NetworkInterfaceName,
}
impl Task for CreateGreInterfaces {
    fn description(&self) -> String {
        String::from("Create GRE interfaces for Peers")
    }
    fn check_fulfilled(&self) -> Result<TaskFulfilled> {
        Ok(TaskFulfilled::Unchecked)
    }
    fn execute(&self) -> Result<Success> {
        block_on(
            gre::remove_existing_interfaces(Arc::clone(&self.network_interface_manager))
        )?;

        let mut netbird_client = block_on(opendut_netbird_client_api::client::Client::connect())?;

        let full_status = block_on(netbird_client.full_status())
            .context("Error during NetBird-Status")?;

        let local_ip = full_status.local_peer_state.unwrap().local_ip()?;

        let router = {
            let mut router = self.router.clone();
            if let Router::Remote(remote_ip) = router {
                if remote_ip == local_ip {
                    log::debug!("Address of Router::Remote '{remote_ip}' is local address. Continuing as Router::Local.");
                    router = Router::Local;
                }
            }
            router
        };

        if let Router::Remote(remote_ip) = router {
            //Create GRE interface to router.
            let interface_index = 0;
            block_on(gre::create_interface(local_ip, remote_ip, interface_index, &self.bridge_name, Arc::clone(&self.network_interface_manager)))?;
            Ok(Success::message(String::from("Interface to router created")))
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
                            log::warn!("Discarding IP address returned by NetBird: {cause}");
                            None
                        }
                    }
                })
                .collect::<Vec<_>>();

            let number_of_remote_ips = remote_ips.len();

            for (interface_index, remote_ip) in remote_ips.into_iter().enumerate() {
                block_on(gre::create_interface(local_ip, remote_ip, interface_index, &self.bridge_name, Arc::clone(&self.network_interface_manager)))?;
            }
            Ok(Success::message(format!("{number_of_remote_ips} interface(s) created; acting as router with IP address '{local_ip}'")))
        }
    }
}
