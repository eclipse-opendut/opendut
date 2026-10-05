use leptos::prelude::*;
use crate::peers::configurator::types::UserPeerDescriptor;

impl UserPeerDescriptor {
    pub fn is_valid(&self) -> bool {
        self.valid_general_tab() && self.valid_devices_tab()
    }

    pub fn valid_general_tab(&self) -> bool {
        self.name.is_right()
            && self.location.is_right()
    }

    pub fn valid_network_tab(&self) -> bool {
        self.network.bridge_name.is_right()
    }

    pub fn valid_devices_tab(&self) -> bool {
        self.devices.iter().all(|device_configuration| {
            let device_configuration = device_configuration.get();
            device_configuration.name.is_right()
                && device_configuration.interface.is_some()
        })
    }
}
