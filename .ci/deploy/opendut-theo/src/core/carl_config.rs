use crate::core::localenv::LOCALENV_SECRETS_ENV_FILE;
use crate::core::project::ProjectRootDir;
use std::path::PathBuf;

#[derive(Debug)]
pub struct CarlConfiguration {
    carl_client_secret: String,
    netbird_password: String,
    netbird_management_client_secret: String,
}

impl CarlConfiguration {
    fn carl_toml_template() -> String {
        include_str!("resources/carl.toml.tmpl").to_string()
    }

    pub fn generate() -> Self {
        load_localenv_secrets();
        let carl_client_secret = std::env::var("OPENDUT_CARL_NETWORK_OIDC_CLIENT_SECRET")
            .unwrap_or_else(|_| panic!("Missing OPENDUT_CARL_NETWORK_OIDC_CLIENT_SECRET in {}", LOCALENV_SECRETS_ENV_FILE));
        let netbird_password = std::env::var("NETBIRD_PASSWORD")
            .unwrap_or_else(|_| panic!("Missing NETBIRD_PASSWORD in environment. Please set manually."));
        let netbird_management_client_secret = std::env::var("NETBIRD_MANAGEMENT_CLIENT_SECRET")
            .unwrap_or_else(|_| panic!("Missing NETBIRD_MANAGEMENT_CLIENT_SECRET in {}", LOCALENV_SECRETS_ENV_FILE));

        Self {
            carl_client_secret,
            netbird_password,
            netbird_management_client_secret,
        }
    }

    pub fn config_toml(&self) -> String {
        let template = Self::carl_toml_template();

        template
            .replace("{localenv_devmode_netbird_password}", &self.netbird_password)
            .replace("{localenv_devmode_carl_client_secret}", &self.carl_client_secret)
            .replace("{localenv_devmode_netbird_management_client_secret}", &self.netbird_management_client_secret)
    }
}



fn load_localenv_secrets() {
    let secrets_file = PathBuf::project_path_buf().join(LOCALENV_SECRETS_ENV_FILE);
    dotenvy::from_path(secrets_file)
        .expect("Could not load localenv secrets file");
}
