use std::convert::TryFrom;
use std::path::PathBuf;
use anyhow::{anyhow, Context};
use axum::extract::{FromRef};
use config::Config;
use serde::Serialize;
use shadow_rs::formatcp;
use url::Url;
use opendut_auth::confidential::config::ConfidentialClientConfigData;


#[derive(Clone)]
pub struct HttpState {
    pub lea_config: LeaConfig,
    pub carl_installation_directory: CarlInstallDirectory,
}

#[derive(Clone, Debug, Serialize)]
pub struct LeaIdentityProviderConfig {
    client_id: String,
    issuer_url: Url,
    scopes: String,
}

const LEA_OIDC_CONFIG_PREFIX: &str = "network.oidc.lea";
impl TryFrom<&Config> for LeaIdentityProviderConfig {
    type Error = anyhow::Error;

    fn try_from(config: &Config) -> anyhow::Result<Self> {

        let client_id = config.get_string(LeaIdentityProviderConfig::CLIENT_ID)
            .map_err(|error| anyhow!("Failed to find configuration for `{}`. {}", LeaIdentityProviderConfig::CLIENT_ID, error))?;
        let issuer = config.get_string(LeaIdentityProviderConfig::ISSUER_URL)
            .map_err(|error| anyhow!("Failed to find configuration for `{}`. {}", LeaIdentityProviderConfig::ISSUER_URL, error))?;

        let issuer_url = Url::parse(&issuer)
            .context(format!("Failed to parse OIDC issuer URL `{}`.", issuer))?;

        let lea_raw_scopes = config.get_string(LeaIdentityProviderConfig::SCOPES)
            .map_err(|error| anyhow!("Failed to find configuration for `{}`. {}", LeaIdentityProviderConfig::SCOPES, error))?;

        let scopes = ConfidentialClientConfigData::parse_scopes(&client_id, lea_raw_scopes).into_iter()
            .map(|scope| scope.to_string())
            .collect::<Vec<_>>()
            .join(" ");  // Required by leptos_oidc

        Ok(Self { client_id, issuer_url, scopes })
    }
}
impl LeaIdentityProviderConfig {
    const CLIENT_ID: &'static str = formatcp!("{LEA_OIDC_CONFIG_PREFIX}.client.id");
    const ISSUER_URL: &'static str = formatcp!("{LEA_OIDC_CONFIG_PREFIX}.issuer.url");
    const SCOPES: &'static str = formatcp!("{LEA_OIDC_CONFIG_PREFIX}.scopes");
}

#[derive(Clone, Serialize)]
pub struct LeaConfig {
    pub(crate) carl_url: Url,
    pub(crate) idp_config: Option<LeaIdentityProviderConfig>,
}

impl FromRef<HttpState> for LeaConfig {
    fn from_ref(app_state: &HttpState) -> Self {
        Clone::clone(&app_state.lea_config)
    }
}

#[derive(Clone, Serialize)]
pub struct CarlInstallDirectory {
    pub path: PathBuf,
}

impl CarlInstallDirectory {
    pub(crate) fn determine() -> anyhow::Result<Self> {
        let path = std::env::current_exe()?
            .parent().ok_or_else(|| anyhow!("Current executable has no parent directory."))?
            .to_owned();
        Ok(Self { path })
    }
}

impl FromRef<HttpState> for CarlInstallDirectory {
    fn from_ref(app_state: &HttpState) -> Self {
        Clone::clone(&app_state.carl_installation_directory)
    }
}
