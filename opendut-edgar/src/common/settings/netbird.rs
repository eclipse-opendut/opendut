use std::fmt::{Display, Formatter};
use config::Config;
use serde::{Deserialize, Serialize};


#[derive(Deserialize)]
pub struct NetbirdClientConfig {
    pub log_level: NetbirdLogLevel,
}
impl NetbirdClientConfig {
    pub fn load_from_config(config: &Config) -> anyhow::Result<Self> {
        let log_level = config.get::<NetbirdLogLevel>(super::key::netbird::client::log::level)?;
        Ok(NetbirdClientConfig { log_level })
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
