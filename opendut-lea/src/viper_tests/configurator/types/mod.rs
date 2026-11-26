pub mod validation;

use std::collections::HashMap;
use opendut_lea_components::{Ior, UserInputValue};
use opendut_model::peer::PeerId;
use opendut_model::viper::{ViperBindingValue, ViperParameterName, ViperSourceId, ViperTestId, ViperTestName, ViperTestRunDescriptor};

pub type SourceSelectionError = String;
pub type SourceSelection = Ior<SourceSelectionError, ViperSourceId>;

pub type PeerSelectionError = String;
pub type PeerSelection = Ior<PeerSelectionError, PeerId>;

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
    #[error("Invalid peer ID")]
    InvalidPeerId,
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
    pub peer: PeerSelection,
    pub parameters: HashMap<ViperParameterName, ViperBindingValueInput>,
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

        let peer = configuration
            .peer
            .right_ok_or(ViperTestMisconfiguration::InvalidPeerId)?;

        let parameters = configuration
            .parameters
            .into_iter()
            .map(|(key, value_input)| {
                value_input
                    .right_ok_or(ViperTestMisconfiguration::InvalidParameterValue)
                    .map(|value| (key, value))
            }).collect::<Result<Vec<_>, _>>()?;

        Ok(ViperTestRunDescriptor {
            id: configuration.id,
            name,
            source,
            peer,
            parameters,
        })
    }
}

impl UserViperTestRunDescriptor {
    pub fn apply_descriptor(&mut self, descriptor: ViperTestRunDescriptor) {
        let ViperTestRunDescriptor {
            id: _,
            name,
            source: viper_source,
            peer,
            parameters,
        } = descriptor;

        self.name = UserInputValue::Right(name.value().to_owned());
        self.viper_source = SourceSelection::Right(viper_source);
        self.peer = PeerSelection::Right(peer);

        let mut configured_parameters: HashMap<ViperParameterName, ViperBindingValueInput> =
            HashMap::new();

        for (key, value) in parameters {
            configured_parameters.insert(key, ViperBindingValueInput::Right(value));
        }

        self.parameters = configured_parameters;
    }
}
