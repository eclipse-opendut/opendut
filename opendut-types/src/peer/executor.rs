use std::fmt;
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use serde::{Deserialize, Serialize};
use strum::EnumIter;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutorDescriptors {
    pub executors: Vec<ExecutorDescriptor>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ExecutorDescriptor {
    Executable,
    #[serde(rename_all = "kebab-case")]
    Container {
        engine: Engine,
        name: ContainerName,
        image: ContainerImage,
        volumes: Vec<ContainerVolume>,
        devices: Vec<ContainerDevice>,
        envs: Vec<ContainerEnvironmentVariable>,
        ports: Vec<ContainerPortSpec>,
        command: ContainerCommand,
        args: Vec<ContainerCommandArgument>,
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, EnumIter)]
#[serde(rename_all = "kebab-case")]
pub enum Engine {
    Docker,
    Podman
}

impl Display for Engine {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Engine::Docker =>  write!(f, "Docker"),
            Engine::Podman =>  write!(f, "Podman"),
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum ContainerName {
    #[default]
    Empty,
    Value(String)
}

impl From<ContainerName> for String {
    fn from(value: ContainerName) -> Self {
        match value {
            ContainerName::Empty => String::new(),
            ContainerName::Value(value) => value
        }
    }
}

impl From<&ContainerName> for String {
    fn from(value: &ContainerName) -> Self {
        match value {
            ContainerName::Empty => String::new(),
            ContainerName::Value(value) => value.to_owned()
        }
    }
}

impl TryFrom<String> for ContainerName {
    type Error = IllegalContainerConfiguration;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.is_empty() {
            Ok(ContainerName::Empty)
        } else {
            Ok(ContainerName::Value(value))
        }
    }
}

impl TryFrom<&str> for ContainerName {
    type Error = IllegalContainerConfiguration;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        ContainerName::try_from(value.to_owned())
    }
}

impl FromStr for ContainerName {
    type Err = IllegalContainerConfiguration;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        ContainerName::try_from(value)
    }
}

impl fmt::Display for ContainerName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", String::from(self))
    }
}


#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ContainerEnvironmentVariable {
    name: String,
    value: String,
}

#[derive(thiserror::Error, Clone, Debug)]
pub enum IllegalContainerEnvironmentVariable{
    #[error("Container env name must not be empty.")]
    EmptyName,
}

impl ContainerEnvironmentVariable {
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Result<Self, IllegalContainerEnvironmentVariable> {
        let name= name.into();
        if name.is_empty() {
            Err(IllegalContainerEnvironmentVariable::EmptyName)
        } else {
            Ok(Self{name, value: value.into()})
        }
    }
    
    pub fn name(&self) -> &str {
        self.name.as_str()
    }
    
    pub fn value(&self) -> &str {
        self.value.as_str()
    }
}

impl From<ContainerEnvironmentVariable> for (String, String) {
    fn from(value: ContainerEnvironmentVariable) -> Self {
        (value.name, value.value)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ContainerImage(String);

impl ContainerImage {
    pub const MIN_LENGTH: usize = 1;

    pub fn value(&self) -> &str {
        &self.0
    }
}

#[derive(thiserror::Error, Clone, Debug)]
pub enum IllegalContainerImage {
    #[error(
    "Container Image '{value}' is too short. Expected at least {expected} characters, got {actual}."
    )]
    TooShort {
        value: String,
        expected: usize,
        actual: usize,
    }
}

impl TryFrom<String> for ContainerImage {
    type Error = IllegalContainerImage;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let length = value.len();
        if length < Self::MIN_LENGTH {
            Err(IllegalContainerImage::TooShort {
                value,
                expected: Self::MIN_LENGTH,
                actual: length,
            })
        } else {
            Ok(Self(value))
        }
    }
}

impl TryFrom<&str> for ContainerImage {
    type Error = IllegalContainerImage;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        ContainerImage::try_from(value.to_owned())
    }
}

impl FromStr for ContainerImage {
    type Err = IllegalContainerImage;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        ContainerImage::try_from(value)
    }
}

impl From<ContainerImage> for String {
    fn from(value: ContainerImage) -> Self {
        value.0
    }
}

impl fmt::Display for ContainerImage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ContainerVolume(String);

impl ContainerVolume {
    pub fn value(&self) -> &str {
        &self.0
    }
}

#[derive(thiserror::Error, Clone, Debug)]
pub enum IllegalContainerVolume{
    #[error("Container volume must not be empty.")]
    Empty,
}

impl TryFrom<String> for ContainerVolume {
    type Error = IllegalContainerVolume;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.is_empty() {
            Err(IllegalContainerVolume::Empty)
        } else {
            Ok(Self(value))
        }
    }
}

impl TryFrom<&str> for ContainerVolume {
    type Error = IllegalContainerVolume;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        ContainerVolume::try_from(value.to_owned())
    }
}

impl FromStr for ContainerVolume {
    type Err = IllegalContainerVolume;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        ContainerVolume::try_from(value)
    }
}

impl From<ContainerVolume> for String {
    fn from(value: ContainerVolume) -> Self {
        value.0
    }
}

impl fmt::Display for ContainerVolume {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ContainerDevice(String);

impl ContainerDevice {
    pub fn value(&self) -> &str {
        &self.0
    }
}

#[derive(thiserror::Error, Clone, Debug)]
pub enum IllegalContainerDevice{
    #[error("Container device must not be empty.")]
    Empty,
}

impl TryFrom<String> for ContainerDevice {
    type Error = IllegalContainerDevice;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.is_empty() {
            Err(IllegalContainerDevice::Empty)
        } else {
            Ok(Self(value))
        }
    }
}

impl TryFrom<&str> for ContainerDevice {
    type Error = IllegalContainerDevice;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        ContainerDevice::try_from(value.to_owned())
    }
}

impl FromStr for ContainerDevice {
    type Err = IllegalContainerDevice;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        ContainerDevice::try_from(value)
    }
}

impl From<ContainerDevice> for String {
    fn from(value: ContainerDevice) -> Self {
        value.0
    }
}

impl fmt::Display for ContainerDevice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ContainerPortSpec(String);

impl ContainerPortSpec{
    pub fn value(&self) -> &str {
        &self.0
    }
}

#[derive(thiserror::Error, Clone, Debug)]
pub enum IllegalContainerPortSpec {
    #[error("Container port specification must not be empty.")]
    Empty,
}

impl TryFrom<String> for ContainerPortSpec {
    type Error = IllegalContainerPortSpec;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.is_empty() {
            Err(IllegalContainerPortSpec::Empty)
        } else {
            Ok(Self(value))
        }
    }
}

impl TryFrom<&str> for ContainerPortSpec {
    type Error = IllegalContainerPortSpec;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        ContainerPortSpec::try_from(value.to_owned())
    }
}

impl FromStr for ContainerPortSpec {
    type Err = IllegalContainerPortSpec;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        ContainerPortSpec::try_from(value)
    }
}

impl From<ContainerPortSpec> for String {
    fn from(value: ContainerPortSpec) -> Self {
        value.0
    }
}

impl fmt::Display for ContainerPortSpec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum ContainerCommand {
    #[default]
    Default,
    Value(String)
}

#[derive(thiserror::Error, Clone, Debug)]
pub enum IllegalContainerCommand {}

impl From<ContainerCommand> for String {
    fn from(value: ContainerCommand) -> Self {
        match value {
            ContainerCommand::Default => String::new(),
            ContainerCommand::Value(value) => value
        }
    }
}

impl From<&ContainerCommand> for String {
    fn from(value: &ContainerCommand) -> Self {
        match value {
            ContainerCommand::Default => String::new(),
            ContainerCommand::Value(value) => value.to_owned()
        }
    }
}

impl TryFrom<String> for ContainerCommand {
    type Error = IllegalContainerCommand;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.is_empty() {
            Ok(ContainerCommand::Default)
        } else {
            Ok(ContainerCommand::Value(value))
        }
    }
}

impl TryFrom<&str> for ContainerCommand {
    type Error = IllegalContainerCommand;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        ContainerCommand::try_from(value.to_owned())
    }
}

impl FromStr for ContainerCommand {
    type Err = IllegalContainerCommand;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        ContainerCommand::try_from(value)
    }
}

impl fmt::Display for ContainerCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", String::from(self))
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ContainerCommandArgument(String);

impl ContainerCommandArgument {
    pub fn value(&self) -> &str {
        &self.0
    }
}

#[derive(thiserror::Error, Clone, Debug)]
pub enum IllegalContainerCommandArgument{
    #[error("Container command argument must not be empty.")]
    Empty,
}

impl TryFrom<String> for ContainerCommandArgument {
    type Error = IllegalContainerCommandArgument;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.is_empty() {
            Err(IllegalContainerCommandArgument::Empty)
        } else {
            Ok(Self(value))
        }
    }
}

impl TryFrom<&str> for ContainerCommandArgument {
    type Error = IllegalContainerCommandArgument;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        ContainerCommandArgument::try_from(value.to_owned())
    }
}

impl FromStr for ContainerCommandArgument {
    type Err = IllegalContainerCommandArgument;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        ContainerCommandArgument::try_from(value)
    }
}

impl From<ContainerCommandArgument> for String {
    fn from(value: ContainerCommandArgument) -> Self {
        value.0
    }
}

impl fmt::Display for ContainerCommandArgument {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(thiserror::Error, Clone, Debug)]
pub enum IllegalContainerConfiguration {}
