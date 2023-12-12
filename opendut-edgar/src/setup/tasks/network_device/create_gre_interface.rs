use std::net::Ipv4Addr;
use std::rc::Rc;
use std::str::FromStr;

use anyhow::{Context, Result};
use futures::executor::block_on;

use opendut_types::topology::InterfaceName;

use crate::service::network_device::manager::NetworkDeviceManager;
use crate::setup::Router;
use crate::setup::task::{Success, Task, TaskFulfilled};

const GRE_INTERFACE_NAME_PREFIX: &str = "gre-opendut";

pub struct CreateGreInterfaces {
    pub network_device_manager: Rc<NetworkDeviceManager>,
    pub router: Router,
    pub bridge_name: InterfaceName,
}
impl Task for CreateGreInterfaces {
    fn description(&self) -> String {
        String::from("Create GRE interfaces for Peers")
    }
    fn check_fulfilled(&self) -> Result<TaskFulfilled> {
        Ok(TaskFulfilled::Unchecked)
    }
    fn execute(&self) -> Result<Success> {
        { //Remove previously created GRE interfaces.
            let interfaces_to_remove = block_on(self.network_device_manager.list_interfaces())?
                .into_iter()
                .filter(|interface| interface.name.name().starts_with(GRE_INTERFACE_NAME_PREFIX));

            for interface in interfaces_to_remove {
                block_on(self.network_device_manager.delete_interface(&interface))?;
            }
        }

        let mut netbird_client = block_on(opendut_netbird_client_api::client::Client::connect())?;

        let full_status = block_on(netbird_client.full_status())
            .context("Error during NetBird-Status")?;

        let local_ip = {
            let local_ip = full_status.local_peer_state.unwrap().ip;
            let local_ip = local_ip.split('/').next() //strip CIDR mask
                .context(format!("Iterator.split() should always return a first element. Did not do so when stripping CIDR mask off of local IP '{local_ip}'."))?;
            Ipv4Addr::from_str(local_ip)
                .context(format!("Local IP returned by NetBird '{local_ip}' could not be parsed."))?
        };

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
            self.create_interface(local_ip, remote_ip, interface_index)?;
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
                self.create_interface(local_ip, remote_ip, interface_index)?
            }
            Ok(Success::message(format!("{number_of_remote_ips} interface(s) created; acting as router with IP address '{local_ip}'")))
        }
    }
}

impl CreateGreInterfaces {
    fn create_interface(&self, local_ip: Ipv4Addr, remote_ip: Ipv4Addr, interface_index: usize) -> Result<()> {
        let interface_prefix = GRE_INTERFACE_NAME_PREFIX;
        let interface_name = InterfaceName::try_from(format!("{}{}", interface_prefix, interface_index))
            .context("Error while constructing GRE interface name")?;

        let gre_interface = block_on(self.network_device_manager.create_gretap_v4_interface(&interface_name, &local_ip, &remote_ip))?;
        log::trace!("Created GRE interface '{gre_interface}'.");
        block_on(self.network_device_manager.set_interface_up(&gre_interface))?;
        log::trace!("Set GRE interface '{interface_name}' to 'up'.");

        let bridge = block_on(self.network_device_manager.try_find_interface(&self.bridge_name))?;
        block_on(self.network_device_manager.join_interface_to_bridge(&gre_interface, &bridge))?;

        Ok(())
    }
}
