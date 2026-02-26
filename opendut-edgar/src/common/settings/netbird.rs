use std::fmt::{Display, Formatter};
use opendut_util::pem::{self, ClientAuth};
use serde::{Deserialize, Serialize};
use tracing::debug;
use opendut_util::settings::LoadedConfig;


pub struct NetbirdClientConfig {
    pub log_level: NetbirdLogLevel,
    pub client_auth: ClientAuth,
    pub config_path_client_cert: String,
    pub config_path_client_key: String,
}
impl NetbirdClientConfig {
    pub fn load_from_config(settings: &LoadedConfig) -> anyhow::Result<Self> {
        let log_level = settings.get::<NetbirdLogLevel>(super::key::netbird::client::log::level)?;

        let client_auth = ClientAuth::load_from_config(
            pem::config_keys::VPN_NETBIRD_CLIENT_TLS_CLIENT_AUTH,
            Some(pem::config_keys::DEFAULT_NETWORK_TLS_CLIENT_AUTH),
            settings
        )?;
        let config_path_client_cert = settings.get_string(super::key::netbird::client::config::keys::mtls::certificate)?;
        let config_path_client_key = settings.get_string(super::key::netbird::client::config::keys::mtls::key)?;
        if let ClientAuth::Enabled { .. } = client_auth {
            debug!("NetBird client TLS client authentication is enabled. Client certificate config path: {config_path_client_cert}, client key config path: {config_path_client_key}");
        }

        Ok(NetbirdClientConfig {
            log_level,
            client_auth,
            config_path_client_cert,
            config_path_client_key,
        })
    }
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all="UPPERCASE")]
pub enum NetbirdLogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}
impl Display for NetbirdLogLevel {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            NetbirdLogLevel::Error => "ERROR",
            NetbirdLogLevel::Warn => "WARN",
            NetbirdLogLevel::Info => "INFO",
            NetbirdLogLevel::Debug => "DEBUG",
            NetbirdLogLevel::Trace => "TRACE",
        };
        write!(f, "{value}")
    }
}
