pub mod validation;

use std::collections::HashMap;
use opendut_lea_components::{Ior, UserInputValue};
use opendut_model::cluster::ClusterId;
use opendut_model::viper::{ViperBindingValue, ViperParameterName, ViperSourceId, ViperTestId, ViperTestName, ViperTestRunDescriptor};

pub type SourceSelectionError = String;
pub type SourceSelection = Ior<SourceSelectionError, ViperSourceId>;

pub type ClusterSelectionError = String;
pub type ClusterSelection = Ior<ClusterSelectionError, ClusterId>;

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
    pub parameters: HashMap<String, UserInputValue>,
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

        let mut parameters = HashMap::new();

        for (key_input, value_input) in configuration.parameters {

            let key = ViperParameterName::try_from(key_input)
                .map_err(|_| ViperTestMisconfiguration::InvalidParameterKey)?;

            let value_string = value_input
                .right_ok_or(ViperTestMisconfiguration::InvalidParameterValue)?;
            let value = parse_binding_value(&value_string);

            parameters.insert(key, value);
        }

        Ok(ViperTestRunDescriptor {
            id: configuration.id,
            name,
            source,
            cluster,
            parameters,
        })
    }
}

fn parse_binding_value(raw: &str) -> ViperBindingValue {
    if raw.eq_ignore_ascii_case("true") {
        ViperBindingValue::BooleanValue(true)
    }
    else if raw.eq_ignore_ascii_case("false") {
        ViperBindingValue::BooleanValue(false)
    }
    else if let Ok(num) = raw.parse::<i64>() {
        ViperBindingValue::NumberValue(num)
    }
    else {
        ViperBindingValue::TextValue(raw.to_owned())
    }
}
