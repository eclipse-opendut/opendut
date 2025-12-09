use std::collections::HashMap;
use opendut_lea_components::UserInputValue;
use opendut_model::cluster::ClusterId;
use opendut_model::viper::{ViperRunDescriptor, ViperRunId, ViperRunName, ViperRunParameterKey, ViperRunParameterValue, ViperSourceId, ViperTestSuiteIdentifier};

#[derive(thiserror::Error, Clone, Debug, Eq, PartialEq, Hash)]
#[allow(clippy::enum_variant_names)] // "all variants have the same prefix: `Invalid`"
pub enum TestMisconfiguration {
    #[error("Invalid test name")]
    InvalidName,
    #[error("Invalid source ID")]
    InvalidSourceId,
    #[error("Invalid test suite")]
    InvalidSuite,
    #[error("Invalid cluster ID")]
    InvalidClusterId,
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
    pub cluster: UserInputValue,
    pub parameters: HashMap<String, UserInputValue>,
    pub is_new: bool,
}

impl UserTestConfiguration {

    pub fn is_valid(&self) -> bool {
        let UserTestConfiguration { id: _, name, source, suite, cluster, parameters, is_new: _ } = self;

        name.is_right()
            && source.is_right()
            && suite.is_right()
            && cluster.is_right()
            && parameters.iter().all(|(_, value)| value.is_right())
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

        let cluster = configuration
            .cluster
            .right_ok_or(TestMisconfiguration::InvalidClusterId)
            .and_then(|cluster_id| {
                ClusterId::try_from(cluster_id)
                    .map_err(|_| TestMisconfiguration::InvalidClusterId)
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
            cluster,
            parameters,
        })
    }
}

fn parse_parameter_value(raw: &str) -> ViperRunParameterValue {
    if raw.eq_ignore_ascii_case("true") {
        ViperRunParameterValue::Boolean(true)
    }
    else if raw.eq_ignore_ascii_case("false") {
        ViperRunParameterValue::Boolean(false)
    }
    else if let Ok(num) = raw.parse::<i64>() {
        ViperRunParameterValue::Number(num)
    }
    else {
        ViperRunParameterValue::Text(raw.to_owned())
    }
}
