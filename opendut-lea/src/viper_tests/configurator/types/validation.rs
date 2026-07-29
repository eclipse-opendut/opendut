use crate::viper_tests::configurator::types::UserViperTestRunDescriptor;

impl UserViperTestRunDescriptor {
    pub fn is_valid(&self) -> bool {
        self.valid_general_tab()
            && self.valid_viper_source_tab()
            && self.valid_parameters_tab()
            && self.valid_peer_tab()
    }

    pub fn valid_general_tab(&self) -> bool {
        self.name.is_right()
    }

    pub fn valid_viper_source_tab(&self) -> bool {
        self.viper_source.is_right()
    }

    pub fn valid_parameters_tab(&self) -> bool {
        self.parameters.iter().all(|(_, parameter_value)| {
            parameter_value.is_right()
        })
    }

    pub fn valid_peer_tab(&self) -> bool {
        self.peer.is_right()
    }
}
