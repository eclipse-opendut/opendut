use serde::Serialize;
use strum::EnumIter;

#[derive(Debug, Clone, clap::ValueEnum, Serialize, EnumIter, PartialEq)]
pub(crate) enum DockerCoreServices {
    Network,
    Carl,
    CarlOnHost,
    Dev,
    Keycloak,
    Edgar,
    Netbird,
    Firefox,
    Telemetry,
    NginxWebdav,
    Postgres,
}

impl DockerCoreServices {
    pub fn as_str(&self) -> &'static str {
        match self {
            DockerCoreServices::Carl => "carl",
            DockerCoreServices::CarlOnHost => "carl-on-host",
            DockerCoreServices::Dev => "dev",
            DockerCoreServices::Keycloak => "keycloak",
            DockerCoreServices::Edgar => "edgar",
            DockerCoreServices::Netbird => "netbird",
            DockerCoreServices::Network => "network",
            DockerCoreServices::Firefox => "firefox",
            DockerCoreServices::Telemetry => "telemetry",
            DockerCoreServices::NginxWebdav => "nginx-webdav",
            DockerCoreServices::Postgres => "postgres",
        }
    }
}

impl std::fmt::Display for DockerCoreServices {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
