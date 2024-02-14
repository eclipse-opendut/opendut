#[derive(Debug)]
pub struct CarlConfiguration {
    netbird_management_url: String,
    netbird_management_ca_path: String,
    netbird_api_key: String,
}

impl CarlConfiguration {
    fn new(netbird_management_url: String, netbird_management_ca_path: String, netbird_api_key: String) -> Self {
        Self {
            netbird_management_url,
            netbird_management_ca_path,
            netbird_api_key,
        }
    }

    fn carl_toml_template() -> String {
        include_str!("resources/carl.toml.tmpl").to_string()
    }

    fn carl_environment_template() -> String {
        include_str!("resources/carl.environment.tmpl").to_string()
    }

    /// This configuration expects the Netbird service to be running in the opendut virtual machine.
    pub fn testenv_in_vm_config(netbird_api_key: String) -> Self {
        Self::new(
            "https://192.168.56.10/api".to_string(),
            "resources/development/tls/insecure-development-ca.pem".to_string(),
            netbird_api_key
        )
    }

    /// This configuration expects the Netbird service to be running in docker on the host system.
    pub fn testenv_on_host_config(netbird_api_key: String) -> Self {
        Self::new(
            "https://192.168.32.211/api".to_string(),
            "resources/development/tls/insecure-development-ca.pem".to_string(),
            netbird_api_key
        )
    }

    pub fn config_toml(&self) -> String {
        let template = Self::carl_toml_template();

        template.replace("{netbird_management_url}", &self.netbird_management_url)
                .replace("{netbird_management_ca_path}", &self.netbird_management_ca_path)
                .replace("{netbird_api_key}", &self.netbird_api_key)
    }
    pub fn config_env_variables(&self) -> String {
        let template = Self::carl_environment_template();

        template.replace("{netbird_management_url}", &self.netbird_management_url)
            .replace("{netbird_management_ca_path}", &self.netbird_management_ca_path)
            .replace("{netbird_api_key}", &self.netbird_api_key)
    }
}
