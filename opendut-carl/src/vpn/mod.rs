use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{anyhow, bail};
use config::Config;
use serde::{Deserialize, Serialize};
use serde::de::IntoDeserializer;
use url::Url;

use opendut_vpn::VpnManagementClient;
use opendut_vpn_netbird::{NetbirdManagementClient, NetbirdManagementClientConfiguration, NetbirdToken};

#[derive(Clone)]
pub enum Vpn {
    Enabled { vpn_client: Arc<dyn VpnManagementClient + Send + Sync> },
    Disabled,
}

pub fn create(settings: &Config) -> anyhow::Result<Vpn> {

    let vpn = settings.get::<VpnConfig>("vpn")?;

    if vpn.enabled {
        match vpn.kind {
            None => unknown_enum_variant(settings, "vpn.kind"),
            Some(VpnKind::Netbird) => {
                let netbird_config = settings.get::<VpnNetbirdConfig>("vpn.netbird")?;
                let base_url = netbird_config.url
                    .ok_or_else(|| anyhow!("No configuration found for: vpn.netbird.base.url"))?;
                let ca = netbird_config.ca
                    .ok_or_else(|| anyhow!("No configuration found for: vpn.netbird.ca"))?;
                let auth_secret = netbird_config.auth.secret
                    .ok_or_else(|| anyhow!("No configuration found for: vpn.netbird.auth.secret"))?;
                let auth_token = match netbird_config.auth.r#type {
                    Some(AuthenticationType::PersonalAccessToken) => NetbirdToken::new_personal_access(auth_secret),
                    Some(AuthenticationType::BearerToken) => unimplemented!("Using a bearer token is not yet supported."),
                    None => return unknown_enum_variant(settings, "vpn.netbird.auth.type"),
                };
                log::debug!("Try to parse VPN configuration.");
                let vpn_client = NetbirdManagementClient::create(
                    NetbirdManagementClientConfiguration {
                        management_url: base_url,
                        authentication_token: Some(auth_token),
                        ca: Some(ca),
                        timeout: Duration::from_millis(netbird_config.timeout.ms),
                        retries: netbird_config.retries,
                    }
                )?;
                Ok(Vpn::Enabled { vpn_client: Arc::new(vpn_client) })
            }
        }
    } else {
        Ok(Vpn::Disabled)
    }
}

fn unknown_enum_variant(settings: &Config, key: &str) -> anyhow::Result<Vpn> {
    let value = settings.get_string(key)?;
    if value.trim().is_empty() {
        bail!("No configuration found for: {key}")
    } else {
        bail!("Unknown {key}: {value}")
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all="kebab-case")]
struct VpnConfig {
    enabled: bool,
    #[serde(deserialize_with = "empty_string_as_none")]
    kind: Option<VpnKind>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all="kebab-case")]
enum VpnKind {
    Netbird,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all="kebab-case")]
struct VpnNetbirdConfig {
    #[serde(deserialize_with = "empty_string_as_none")]
    url: Option<Url>,
    #[serde(deserialize_with = "empty_string_as_none")]
    ca: Option<PathBuf>,
    auth: AuthConfig,
    timeout: Timeout,
    retries: u32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all="kebab-case")]
struct Timeout {
    ms: u64,
}


#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all="kebab-case")]
struct AuthConfig {
    #[serde(deserialize_with = "empty_string_as_none")]
    r#type: Option<AuthenticationType>,
    #[serde(deserialize_with = "empty_string_as_none")]
    secret: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all="kebab-case")]
enum AuthenticationType {
    PersonalAccessToken,
    BearerToken,
}

fn empty_string_as_none<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: serde::Deserialize<'de>,
{
    let maybe_string = Option::<String>::deserialize(deserializer)?;
    match maybe_string.as_deref() {
        None | Some("") => Ok(None),
        Some(string) => T::deserialize(string.into_deserializer()).map(Some)
    }
}
