use crate::compile::ParameterName;

#[derive(Clone, Debug)]
#[cfg_attr(test, derive(PartialEq))]
#[allow(dead_code)]
#[non_exhaustive]
pub enum BindParameterError {
    ParameterNotFound(ParameterName),
    TypeMismatch {
        parameter_name: ParameterName,
        expected_type: String,
        actual_type: String
    },
    NumberValueOutOfRange {
        parameter_name: ParameterName,
        value: i64,
        min: i64,
        max: i64
    },
    TextValueOutOfRange {
        parameter_name: ParameterName,
        value: String,
        max: u16
    },
}

impl BindParameterError {

    pub fn new_parameter_not_found_error(name: impl Into<ParameterName>) -> Self {
        Self::ParameterNotFound(name.into())
    }

    pub fn new_type_mismatch_error(name: impl Into<ParameterName>, expected: impl Into<String>, actual: impl Into<String>) -> Self {
        Self::TypeMismatch {
            parameter_name: name.into(),
            expected_type: expected.into(),
            actual_type: actual.into()
        }
    }

    pub fn new_number_value_out_of_range_error(name: impl Into<ParameterName>, value: i64, min: i64, max: i64) -> Self {
        Self::NumberValueOutOfRange {
            parameter_name: name.into(),
            value,
            min,
            max
        }
    }

    pub fn new_text_value_out_of_range_error(name: impl Into<ParameterName>, value: impl Into<String>, max: u16) -> Self {
        Self::TextValueOutOfRange {
            parameter_name: name.into(),
            value: value.into(),
            max,
        }
    }
}

#[derive(Clone, Debug)]
#[cfg_attr(test, derive(PartialEq))]
#[allow(dead_code)]
#[non_exhaustive]
pub struct IncompleteParameterBindingsError {
    pub missing_parameters: Vec<ParameterName>
}

impl IncompleteParameterBindingsError {

    pub(crate) fn new(missing_parameters: Vec<ParameterName>) -> Self {
        Self {
            missing_parameters
        }
    }
}
