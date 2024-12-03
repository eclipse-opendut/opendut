
use std::fmt;
use std::str::FromStr;
use serde::{Deserialize, Serialize};
use url::Url;
use uuid::Uuid;
use crate::peer::executor::container::{Engine, ContainerName, ContainerImage, ContainerVolume, ContainerDevice, ContainerEnvironmentVariable, ContainerPortSpec, ContainerCommand, ContainerCommandArgument, deserialize_container_environment_variable_vec};

pub mod container;

#[derive(Clone, Debug, PartialEq,  Eq, Serialize, Deserialize)]
pub struct ExecutorDescriptors {
    pub executors: Vec<ExecutorDescriptor>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct ExecutorDescriptor {
    pub id: ExecutorId,
    #[serde(flatten)]
    pub kind: ExecutorKind,
    pub results_url: Option<ResultsUrl>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ExecutorId { pub uuid: Uuid }
impl ExecutorId {
    pub fn random() -> Self {
        Self { uuid: Uuid::new_v4() }
    }
}
impl From<Uuid> for ExecutorId {
    fn from(uuid: Uuid) -> Self {
        Self { uuid }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub enum ExecutorKind {
    Executable,
    #[serde(rename_all = "kebab-case")]
    Container {
        engine: Engine,
        #[serde(default)]
        name: ContainerName,
        image: ContainerImage,
        volumes: Vec<ContainerVolume>,
        #[serde(default)]
        devices: Vec<ContainerDevice>,
        #[serde(default, deserialize_with = "deserialize_container_environment_variable_vec")]
        envs: Vec<ContainerEnvironmentVariable>,
        #[serde(default)]
        ports: Vec<ContainerPortSpec>,
        #[serde(default)]
        command: ContainerCommand,
        #[serde(default)]
        args: Vec<ContainerCommandArgument>,
    }
}



#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct ResultsUrl(Url);

impl ResultsUrl {
    pub fn value(&self) -> &Url {
        &self.0
    }
}

#[derive(thiserror::Error, Clone, Debug)]
pub enum IllegalResultsUrl{
    #[error("Failed to parse results URL: {cause}")]
    ParseFailure {cause: url::ParseError},
}

impl TryFrom<&str> for ResultsUrl {
    type Error = IllegalResultsUrl;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match Url::parse(value) {
            Ok(url) => Ok(Self(url)),
            Err(cause) => Err(IllegalResultsUrl::ParseFailure { cause}),
        }
    }
}

impl TryFrom<String> for ResultsUrl {
    type Error = IllegalResultsUrl;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        ResultsUrl::try_from(value.as_str())
    }
}

impl FromStr for ResultsUrl {
    type Err = IllegalResultsUrl;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        ResultsUrl::try_from(value)
    }
}

impl From<ResultsUrl> for String {
    fn from(value: ResultsUrl) -> Self {
        value.0.to_string()
    }
}

impl fmt::Display for ResultsUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
