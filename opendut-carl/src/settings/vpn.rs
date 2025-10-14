use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{anyhow, bail};
use config::Config;
use tracing::debug;
use url::Url;
use opendut_auth::confidential::config::{OAuthClientId, OAuthClientSecret, OidcConfidentialClientConfig, OidcResourceOwnerConfidentialClientConfig};
use opendut_auth::confidential::IssuerUrl;
use opendut_vpn::VpnManagementClient;
use opendut_vpn_netbird::{NetbirdAuthenticationMethod, NetbirdManagementClient, NetbirdManagementClientConfiguration, NetbirdToken};

#[derive(Clone)]
pub enum Vpn {
    Enabled { vpn_client: Arc<dyn VpnManagementClient + Send + Sync> },
    Disabled,
}

pub async fn create(settings: &Config) -> anyhow::Result<Vpn> {

    let vpn = settings.get::<bool>("vpn.enabled")?;

    if vpn {
        let vpn_kind_key = "vpn.kind";
        let vpn_kind = settings.get::<String>(vpn_kind_key)?;

        match vpn_kind.as_ref() {
            "netbird" => {
                let base_url = settings.get::<Option<Url>>("vpn.netbird.url")?
                    .ok_or_else(|| anyhow!("No configuration found for: vpn.netbird.url"))?;

                let ca = settings.get::<Option<PathBuf>>("vpn.netbird.ca")?
                    .ok_or_else(|| anyhow!("No configuration found for: vpn.netbird.ca"))?;

                let auth_secret = settings.get::<Option<String>>("vpn.netbird.auth.secret")?
                    .ok_or_else(|| anyhow!("No configuration found for: vpn.netbird.auth.secret"))?;

                let vpn_netbird_auth_type = "vpn.netbird.auth.type";
                let auth_type = settings.get::<Option<String>>(vpn_netbird_auth_type)?;

                let authentication = match auth_type {
                    Some(auth_type) => match auth_type.as_ref() {
                        "personal-access-token" => NetbirdAuthenticationMethod::UseExistingApiToken(NetbirdToken::new_personal_access(auth_secret)),
                        "bearer-token" => unimplemented!("Using a bearer token is not yet supported."),
                        "oauth-create-api-token" => {
                            let username = settings.get::<Option<String>>("vpn.netbird.auth.username")?
                                .ok_or_else(|| anyhow!("No configuration found for: vpn.netbird.auth.username"))?;
                            let password = settings.get::<Option<String>>("vpn.netbird.auth.password")?
                                .ok_or_else(|| anyhow!("No configuration found for: vpn.netbird.auth.password"))?;

                            let issuer_url = settings.get::<Option<String>>("vpn.netbird.auth.oidc.issuer")?
                                .ok_or_else(|| anyhow!("No configuration found for: vpn.netbird.auth.oidc.issuer"))?;
                            let client_id = settings.get::<Option<String>>("vpn.netbird.auth.oidc.id")?
                                .ok_or_else(|| anyhow!("No configuration found for: vpn.netbird.auth.oidc.id"))?;
                            let client_secret = settings.get::<Option<String>>("vpn.netbird.auth.oidc.secret")?
                                .ok_or_else(|| anyhow!("No configuration found for: vpn.netbird.auth.oidc.secret"))?;
                            let scopes = settings.get::<Option<String>>("vpn.netbird.auth.oidc.scopes")?
                                .ok_or_else(|| anyhow!("No configuration found for: vpn.netbird.auth.oidc.scopes"))?;
                            let scopes = OidcConfidentialClientConfig::parse_scopes(&client_id, scopes);

                            // OIDC credentials used for resource owner password credential grant
                            let client_id = OAuthClientId::new(client_id);
                            let client_secret = OAuthClientSecret::new(client_secret);
                            let issuer_url = IssuerUrl::try_from(issuer_url.as_str())?;
                            NetbirdAuthenticationMethod::CreateApiTokenWithOidc(OidcResourceOwnerConfidentialClientConfig::new(
                                client_id,
                                client_secret,
                                issuer_url,
                                scopes,
                                username,
                                password,
                            ))

                        },
                        _ => return Err(anyhow!("Invalid configuration parameter for '{vpn_netbird_auth_type}', allowed values are 'bearer-token' and 'personal-access-token'.")),
                    }
                    None => return unknown_enum_variant(settings, vpn_netbird_auth_type)
                };

                let timeout_ms = settings.get::<Option<u64>>("vpn.netbird.timeout.ms")?
                    .ok_or_else(|| anyhow!("No configuration found for: vpn.netbird.timeout.ms"))?;

                let retries = settings.get::<Option<u32>>("vpn.netbird.retries")?
                    .ok_or_else(|| anyhow!("No configuration found for: vpn.netbird.retries"))?;

                let setup_key_expiration_ms = settings.get::<Option<u64>>("vpn.netbird.setup.key.expiration.ms")?
                    .ok_or_else(|| anyhow!("No configuration found for: vpn.netbird.setup.key.expiration.ms"))?;
                
                debug!("Try to parse VPN configuration.");
                let vpn_client = NetbirdManagementClient::create_client_and_delete_default_policy(
                    NetbirdManagementClientConfiguration {
                        management_url: base_url,
                        authentication,
                        ca: Some(ca),
                        timeout: Duration::from_millis(timeout_ms),
                        retries,
                        setup_key_expiration: Duration::from_millis(setup_key_expiration_ms),
                    }
                ).await?;
                Ok(Vpn::Enabled { vpn_client: Arc::new(vpn_client) })
            }
            "" => unknown_enum_variant(settings, vpn_kind_key),
            other => Err(anyhow!("Invalid configuration parameter '{other}' for key '{vpn_kind_key}', allowed value is 'netbird'.")),
        }
    } else {
        Ok(Vpn::Disabled)
    }
}

fn unknown_enum_variant(settings: &Config, key: &str) -> anyhow::Result<Vpn> {
    let value = settings.get_string(key)?;
    if value.trim().is_empty() {
        bail!("No configuration found for: {key}")
    }
    bail!("Unknown {key}: {value}")
}
