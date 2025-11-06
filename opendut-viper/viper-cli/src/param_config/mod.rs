mod error;

use std::collections::HashMap;
use serde::Deserialize;
use opendut_viper_rt::compile::{ParameterDescriptor, ParameterName};
use opendut_viper_rt::run::{BindParameterError, BindingValue, Incomplete, ParameterBindings};

pub use error::{
    IncompleteBindingsError,
    ParameterTomlError,
};

#[derive(Debug, Default)]
pub struct ParameterToml {
    pub test_suite_parameters: Vec<TestSuiteParameters>,
}

#[derive(Debug)]
pub struct TestSuiteParameters {
    pub test_suite: String,
    pub parameters: Vec<Parameter>,
}

#[derive(Clone, Debug)]
pub struct Parameter {
    pub name: String,
    pub value: toml::Value,
}

#[derive(Deserialize)]
pub struct RawParameterToml {
    #[serde(flatten)]
    test_suites: HashMap<String, HashMap<String, toml::Value>>,
}

impl ParameterToml {
    pub fn load(params_from_file: &Option<String>) -> Result<Self, Box<dyn std::error::Error>> {
        if let Some(path) = params_from_file {
            let content = std::fs::read_to_string(path)?;
            let raw_config: RawParameterToml = toml::from_str(&content)?;
            Ok(raw_config.into())
        } else {
            Ok(Self::default())
        }
    }

    pub fn bind_parameters_for_suite(
        &self,
        suite_name: &str,
        bindings: &mut ParameterBindings<Incomplete>
    ) -> Result<(), ParameterTomlError> {

        bindings.bind_each(|descriptor| {

            let suite_params = self.test_suite_parameters
                .iter()
                .find(|toml_suite_params| toml_suite_params.test_suite == suite_name)
                .ok_or_else(|| BindParameterError::new_parameter_not_found_error(descriptor))?;

            let parameter = suite_params.parameters
                .iter()
                .find(|param| {
                    let Ok(name) = ParameterName::try_from(param.name.clone()) else { return false };
                    descriptor.name() == &name
                })
                .ok_or_else(|| BindParameterError::new_parameter_not_found_error(descriptor))?;

            Self::determine_binding_value(descriptor, parameter)

        }).map_err(|err| ParameterTomlError {
            suite: suite_name.to_owned(),
            cause: err,
        })
    }

    fn determine_binding_value(descriptor: &ParameterDescriptor, parameter: &Parameter)
        -> Result<BindingValue, BindParameterError>{

        let parameter_name = ParameterName::try_from(parameter.name.clone())
            .expect("Failed to parse suite parameter name");

        let parameter_value = &parameter.value;

        match (descriptor, parameter_value) {
            (ParameterDescriptor::BooleanParameter { .. }, toml::Value::Boolean(value)) => {
                Ok(BindingValue::BooleanValue(value.to_owned()))
            }
            (ParameterDescriptor::NumberParameter { .. }, toml::Value::Integer(value)) => {
                Ok(BindingValue::NumberValue(value.to_owned()))
            }
            (ParameterDescriptor::TextParameter { .. }, toml::Value::String(value)) => {
                Ok(BindingValue::TextValue(value.to_owned()))
            }
            (expected, actual) => {
                let expected_type = match expected {
                    ParameterDescriptor::BooleanParameter { .. } => "Boolean",
                    ParameterDescriptor::NumberParameter { .. }  => "Integer",
                    ParameterDescriptor::TextParameter { .. }    => "String",
                };

                Err(BindParameterError::new_type_mismatch_error(
                    parameter_name,
                    expected_type.to_string(),
                    Self::toml_type_name(actual).to_string(),
                ))
            }
        }
    }

    fn toml_type_name(value: &toml::Value) -> &'static str {
        match value {
            toml::Value::String(_) => "String",
            toml::Value::Integer(_) => "Integer",
            toml::Value::Float(_) => "Float",
            toml::Value::Boolean(_) => "Boolean",
            toml::Value::Datetime(_) => "Datetime",
            toml::Value::Array(_) => "Array",
            toml::Value::Table(_) => "Table",
        }
    }
}

impl From<RawParameterToml> for ParameterToml {
    fn from(raw: RawParameterToml) -> Self {
        let mut test_suite_parameters = Vec::new();

        for (suite, params) in raw.test_suites {

            let converted_parameters = params.iter()
                .map(|(key, value)| {
                    Parameter { name: key.to_owned(), value: value.to_owned() }
                })
                .collect::<Vec<Parameter>>();

            test_suite_parameters.push(TestSuiteParameters {
                test_suite: suite,
                parameters: converted_parameters
            })
        }

        ParameterToml { test_suite_parameters }
    }
}
