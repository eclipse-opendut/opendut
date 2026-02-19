use crate::viper_sources::configurator::types::UserViperSourceConfiguration;

impl UserViperSourceConfiguration {
    pub fn is_valid(&self) -> bool {
        self.name.is_right()
            && self.url.is_right()
    }
}
