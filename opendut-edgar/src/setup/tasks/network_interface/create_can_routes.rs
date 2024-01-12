use std::rc::Rc;
use std::collections::HashSet;
use itertools::Itertools;

use anyhow::Result;
use futures::executor::block_on;
use config::Config;
use opendut_types::topology::{InterfaceName, Topology};

use crate::common::settings;
use crate::service::network_device::manager::NetworkDeviceManager;
use crate::setup::task::{Success, Task, TaskFulfilled};

pub struct SetupLocalCanRouting {
    pub network_device_manager: Rc<NetworkDeviceManager>,
    pub vcan_interface_name: InterfaceName,
}

impl SetupLocalCanRouting {
    pub async fn get_can_interfaces(&self) -> Result<HashSet<InterfaceName>>{
        let settings = settings::load_with_overrides(Config::default()).expect("Failed to load configuration.");

        // TODO: What if the topology doesn't exist in the config?
        let topology = settings.config
            .get::<Topology>("topology")
            .expect("Unable to load topology from configuration");

        let mut can_interfaces: HashSet<InterfaceName> = topology.devices
            .into_iter()
            .filter(|device| device.tags.iter().any(|tag| tag == "CAN"))
            .map(|device| device.interface)
            .collect::<HashSet<_>>();

        can_interfaces.insert(self.vcan_interface_name.clone());

        Ok(can_interfaces)
    }
}
impl Task for SetupLocalCanRouting {
    fn description(&self) -> String {
        format!("Setup local CAN routing with vcan interface \"{}\"", self.vcan_interface_name)
    }
    fn check_fulfilled(&self) -> Result<TaskFulfilled> {
        if block_on(self.network_device_manager.find_interface(&self.vcan_interface_name))?.is_none() {
            return Ok(TaskFulfilled::No)
        }

        let can_interfaces = block_on(self.get_can_interfaces())?;

        for permutation in can_interfaces.iter().permutations(2) {
            let [src, dst] = permutation.try_into().unwrap();
            if ! block_on(self.network_device_manager.check_can_route_exists((*src).clone(), (*dst).clone(), true))? {
                return Ok(TaskFulfilled::No)
            }
        }

        Ok(TaskFulfilled::Yes)
    }
    fn execute(&self) -> Result<Success> {
        
        let bridge = block_on(
            self.network_device_manager
                .create_vcan_interface(&self.vcan_interface_name),
        )?;
        block_on(self.network_device_manager.set_interface_up(&bridge))?;

        block_on(self.network_device_manager.load_cangw_kernel_module())?;
        block_on(self.network_device_manager.remove_all_can_routes())?;

        let can_interfaces = block_on(self.get_can_interfaces())?;

        for permutation in can_interfaces.iter().permutations(2) {
            let [src, dst] = permutation.try_into().unwrap();
            block_on(self.network_device_manager.create_can_route((*src).clone(), (*dst).clone(), false))?;
            block_on(self.network_device_manager.create_can_route((*src).clone(), (*dst).clone(), true))?;
        }

        Ok(Success::default())
    }
}
