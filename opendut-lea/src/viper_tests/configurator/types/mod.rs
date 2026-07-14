pub mod validation;

use std::collections::HashMap;
use opendut_lea_components::{Ior, UserInputValue};
use opendut_model::cluster::ClusterId;
use opendut_model::peer::PeerId;
use opendut_model::viper::{ViperBindingValue, ViperParameterName, ViperSourceId, ViperTestId, ViperTestName, ViperTestRunDescriptor};

pub type SourceSelectionError = String;
pub type SourceSelection = Ior<SourceSelectionError, ViperSourceId>;

pub type ClusterSelectionError = String;
pub type ClusterSelection = Ior<ClusterSelectionError, ClusterId>;

pub type ViperBindingValueError = String;
pub type ViperBindingValueInput = Ior<ViperBindingValueError, Option<ViperBindingValue>>;

#[derive(thiserror::Error, Clone, Debug, Eq, PartialEq, Hash)]
#[allow(clippy::enum_variant_names)] // "all variants have the same prefix: `Invalid`"
pub enum ViperTestMisconfiguration {
    #[error("Invalid viper test name")]
    InvalidName,
    #[error("Invalid viper source ID")]
    InvalidSourceId,
    #[error("Invalid viper test suite")]
    InvalidSuite,
    #[error("Invalid cluster ID")]
    InvalidClusterId,
    #[error("Invalid viper test parameter key")]
    InvalidParameterKey,
    #[error("Invalid viper test parameter value")]
    InvalidParameterValue,
}

#[derive(Clone, Debug)]
pub struct UserViperTestRunDescriptor {
    pub id: ViperTestId,
    pub name: UserInputValue,
    pub viper_source: SourceSelection,
    pub cluster: ClusterSelection,
    pub parameters: HashMap<ViperParameterName, ViperBindingValueInput>,
    pub is_new: bool,
}

impl TryFrom<UserViperTestRunDescriptor> for ViperTestRunDescriptor {
    type Error = ViperTestMisconfiguration;

    fn try_from(configuration: UserViperTestRunDescriptor) -> Result<Self, Self::Error> {
        let name = configuration
            .name
            .right_ok_or(ViperTestMisconfiguration::InvalidName)
            .and_then(|name| {
                ViperTestName::try_from(name)
                    .map_err(|_| ViperTestMisconfiguration::InvalidName)
            })?;

        let source = configuration
            .viper_source
            .right_ok_or(ViperTestMisconfiguration::InvalidSourceId)?;

        let cluster = configuration
            .cluster
            .right_ok_or(ViperTestMisconfiguration::InvalidClusterId)?;

        let peer = PeerId::random();

        let mut parameters = HashMap::new();

        for (key, value_input) in configuration.parameters {

            let value = value_input
                .right_ok_or(ViperTestMisconfiguration::InvalidParameterValue)?;

            parameters.insert(key, value);
        }

        Ok(ViperTestRunDescriptor {
            id: configuration.id,
            name,
            source,
            cluster,
            peer,
            parameters,
        })
    }
}
