use crate::clusters::configurator::types::UserClusterDescriptor;

impl UserClusterDescriptor {
    pub fn is_valid(&self) -> bool {
        self.valid_general_tab()
            && self.valid_devices_tab()
            && self.valid_leader_tab()
    }

    pub fn valid_general_tab(&self) -> bool {
        self.name.is_right()
    }

    pub fn valid_devices_tab(&self) -> bool {
        self.devices.is_right()
    }

    pub fn valid_leader_tab(&self) -> bool {
        self.leader.is_right()
    }
}
