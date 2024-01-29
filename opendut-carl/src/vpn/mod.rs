use std::str::FromStr;
use std::sync::Arc;

use anyhow::bail;
use config::Config;
use serde::{de, Deserialize, Deserializer, Serialize};
use serde::de::IntoDeserializer;
use url::Url;

use opendut_types::vpn::HttpsOnly;
use opendut_vpn::VpnManagementClient;
use opendut_vpn_netbird::{DefaultVpnManagementClient, NetbirdToken};

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

                match netbird_config.url {
                    None => bail!("No configuration found for: vpn.netbird.base.url"),
                    Some(base_url) => {
                        match netbird_config.auth.secret {
                            None => bail!("No configuration found for: vpn.netbird.auth.secret"),
                            Some(auth_secret) => {

                                let token = match netbird_config.auth.r#type {
                                    Some(AuthenticationType::PersonalAccessToken) => NetbirdToken::new_personal_access(auth_secret),
                                    Some(AuthenticationType::BearerToken) => unimplemented!("Using a bearer token is not yet supported."),
                                    None => return unknown_enum_variant(settings, "vpn.netbird.auth.type"),
                                };

                                let vpn_client = DefaultVpnManagementClient::create(
                                    base_url,
                                    token,
                                )?;

                                Ok(Vpn::Enabled { vpn_client: Arc::new(vpn_client) })
                            }
                        }
                    }
                }
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

fn deserialize_https_only_from_str<'de, D>(deserializer: D) -> Result<HttpsOnly, D::Error>
    where D: Deserializer<'de>
{
    let s = String::deserialize(deserializer)?;
    let result = bool::from_str(&s).map_err(de::Error::custom)?;
    match result {
        true => Ok(HttpsOnly::True),
        false => Ok(HttpsOnly::False),
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all="kebab-case")]
struct VpnNetbirdConfig {
    #[serde(deserialize_with = "empty_string_as_none")]
    url: Option<Url>,
    https: Https,
    auth: AuthConfig,
}

#[derive(Debug, Serialize, Deserialize)]
struct Https {
    #[serde(default = "HttpsOnly::default", deserialize_with = "deserialize_https_only_from_str")]
    only: HttpsOnly
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
