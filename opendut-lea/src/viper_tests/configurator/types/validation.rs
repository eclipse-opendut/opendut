use crate::viper_tests::configurator::types::UserViperTestConfiguration;

impl UserViperTestConfiguration {
    pub fn is_valid(&self) -> bool {
        self.valid_general_tab()
            && self.valid_viper_source_tab()
            && self.valid_parameters_tab()
            && self.valid_cluster_tab()
    }

    pub fn valid_general_tab(&self) -> bool {
        self.name.is_right()
    }

    pub fn valid_viper_source_tab(&self) -> bool {
        self.viper_source.is_right()
    }

    pub fn valid_parameters_tab(&self) -> bool {
        self.parameters.iter().all(|parameter| {
            parameter.1.is_right()
        })
    }

    pub fn valid_cluster_tab(&self) -> bool {
        self.cluster.is_right()
    }
}
