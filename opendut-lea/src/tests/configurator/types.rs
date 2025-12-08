use std::collections::HashMap;
use opendut_lea_components::UserInputValue;
use opendut_model::viper::{ViperRunDescriptor, ViperRunId, ViperRunName, ViperRunParameterKey, ViperRunParameterValue, ViperSourceId, ViperTestSuiteIdentifier};

#[derive(thiserror::Error, Clone, Debug, Eq, PartialEq, Hash)]
#[allow(clippy::enum_variant_names)]
pub enum TestMisconfiguration {
    #[error("Invalid test name")]
    InvalidName,
    #[error("Invalid source id")]
    InvalidSourceId,
    #[error("Invalid test suite")]
    InvalidSuite,
    #[error("Invalid test parameter key")]
    InvalidParameterKey,
    #[error("Invalid test parameter value")]
    InvalidParameterValue,
}

#[derive(Clone, Debug)]
pub struct UserTestConfiguration {
    pub id: ViperRunId,
    pub name: UserInputValue,
    pub source: UserInputValue,
    pub suite: UserInputValue,
    pub parameters: HashMap<String, UserInputValue>,
    pub is_new: bool,
}

impl UserTestConfiguration {

    pub fn is_valid(&self) -> bool {
        self.name.is_right()
            && self.source.is_right()
            && self.suite.is_right()
            && self.parameters.iter().all(|(_,v)| v.is_right())
    }
}

impl TryFrom<UserTestConfiguration> for ViperRunDescriptor {
    type Error = TestMisconfiguration;

    fn try_from(configuration: UserTestConfiguration) -> Result<Self, Self::Error> {
        let name = configuration
            .name
            .right_ok_or(TestMisconfiguration::InvalidName)
            .and_then(|name| {
                ViperRunName::try_from(name)
                    .map_err(|_| TestMisconfiguration::InvalidName)
            })?;

        let source = configuration
            .source
            .right_ok_or(TestMisconfiguration::InvalidSourceId)
            .and_then(|source_id| {
                ViperSourceId::try_from(source_id)
                    .map_err(|_| TestMisconfiguration::InvalidSourceId)
            })?;

        let suite = configuration
            .suite
            .right_ok_or(TestMisconfiguration::InvalidSuite)
            .and_then(|suite_id| {
                ViperTestSuiteIdentifier::try_from(suite_id)
                    .map_err(|_| TestMisconfiguration::InvalidSuite)
            })?;

        let mut parameters = HashMap::new();

        for (key_input, value_input) in configuration.parameters {

            let key = ViperRunParameterKey {
                inner: key_input,
            };

            let value_string = value_input
                .right_ok_or(TestMisconfiguration::InvalidParameterValue)?;
            let value = parse_parameter_value(&value_string);

            parameters.insert(key, value);
        }

        Ok(ViperRunDescriptor {
            id: configuration.id,
            name,
            source,
            suite,
            parameters,
        })
    }
}

fn parse_parameter_value(raw: &str) -> ViperRunParameterValue {
    if raw.eq_ignore_ascii_case("true") {
        ViperRunParameterValue::Boolean(true)
    } else if raw.eq_ignore_ascii_case("false") {
        ViperRunParameterValue::Boolean(false)
    } else if let Ok(num) = raw.parse::<i64>() {
        ViperRunParameterValue::Number(num)
    } else {
        ViperRunParameterValue::Text(raw.to_owned())
    }
}
