use crate::sources::configurator::types::UserSourceConfiguration;

impl UserSourceConfiguration {
    pub fn is_valid(&self) -> bool {
        self.name.is_right()
            && self.url.is_right()
    }
}
