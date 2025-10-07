#[derive(Debug)]
pub struct CarlConfiguration {
    netbird_api_key: String,
}

impl CarlConfiguration {
    fn new(netbird_api_key: String) -> Self {
        Self {
            netbird_api_key,
        }
    }

    fn carl_toml_template() -> String {
        include_str!("resources/carl.toml.tmpl").to_string()
    }

    pub fn generate(netbird_api_key: String) -> Self {
        Self::new(
            netbird_api_key,
        )
    }

    pub fn config_toml(&self) -> String {
        let template = Self::carl_toml_template();

        template.replace("{netbird_api_key}", &self.netbird_api_key)
    }
}
