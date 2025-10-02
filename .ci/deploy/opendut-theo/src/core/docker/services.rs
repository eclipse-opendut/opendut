use serde::Serialize;
use strum::EnumIter;

#[derive(Debug, Clone, clap::ValueEnum, Serialize, EnumIter, PartialEq)]
pub(crate) enum DockerCoreServices {
    Carl,
    CarlOnHost,
    Edgar,
    Firefox,
}

impl DockerCoreServices {
    pub fn as_str(&self) -> &'static str {
        match self {
            DockerCoreServices::Carl => "carl",
            DockerCoreServices::CarlOnHost => "carl-on-host",
            DockerCoreServices::Edgar => "edgar",
            DockerCoreServices::Firefox => "firefox",
        }
    }
}

impl std::fmt::Display for DockerCoreServices {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
