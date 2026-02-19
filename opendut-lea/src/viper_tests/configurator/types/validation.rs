use crate::viper_tests::configurator::types::UserTestConfiguration;

impl UserTestConfiguration {
    pub fn is_valid(&self) -> bool {
        self.valid_general_tab()
            && self.valid_source_tab()
            && self.valid_suite_tab()
            && self.valid_cluster_tab()
    }

    pub fn valid_general_tab(&self) -> bool {
        self.name.is_right()
    }

    pub fn valid_source_tab(&self) -> bool {
        self.source.is_right()
    }

    pub fn valid_suite_tab(&self) -> bool {
        self.suite.is_right()
    }

    pub fn valid_cluster_tab(&self) -> bool {
        self.cluster.is_right()
    }
}
