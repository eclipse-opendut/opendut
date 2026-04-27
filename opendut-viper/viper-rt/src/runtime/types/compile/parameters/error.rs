#[derive(Debug)]
#[allow(dead_code)]
#[non_exhaustive]
pub enum ParameterError {
    IllegalParameterName(InvalidParameterNameError),
    IllegalParameterValue(InvalidParameterValueError),
}

#[derive(Debug)]
pub struct InvalidParameterValueError {
    pub value: String,
    pub kind: InvalidParameterValueErrorKind,
}

impl InvalidParameterValueError {
    pub fn new_empty_parameter_value_error() -> Self {
        Self { value: String::new(), kind: InvalidParameterValueErrorKind::Empty }
    }
    
    pub fn new_too_long_parameter_value_error(value: impl Into<String>, expected: usize, actual: usize) -> Self {
        Self { value: value.into(), kind: InvalidParameterValueErrorKind::TooLong { expected, actual } }
    }

    pub fn new_invalid_type_parameter_value_error(value: impl Into<String>, expected: String, actual: String) -> Self {
        Self { value: value.into(), kind: InvalidParameterValueErrorKind::InvalidType { expected, actual}}
    }
}

#[derive(Debug)]
pub enum InvalidParameterValueErrorKind {
    Empty,
    TooLong { expected: usize, actual: usize },
    InvalidType { expected: String, actual: String },
}

#[derive(Debug)]
#[cfg_attr(any(test, doc), derive(PartialEq))]
#[allow(dead_code)]
#[non_exhaustive]
pub struct InvalidParameterNameError {
    pub value: String,
    pub kind: InvalidParameterNameErrorKind,
}

impl InvalidParameterNameError {

    pub fn new_empty_parameter_name_error() -> Self {
        Self { value: String::new(), kind: InvalidParameterNameErrorKind::Empty }
    }

    pub fn new_illegal_parameter_name_character_error(value: impl Into<String>, character: char) -> Self {
        Self { value: value.into(), kind: InvalidParameterNameErrorKind::IllegalCharacter { character } }
    }
}

#[derive(Debug)]
#[cfg_attr(any(test, doc), derive(PartialEq))]
#[allow(dead_code)]
#[non_exhaustive]
pub enum InvalidParameterNameErrorKind {
    Empty,
    IllegalCharacter { character: char },
}

impl From<InvalidParameterNameError> for ParameterError {
    fn from(value: InvalidParameterNameError) -> Self {
        ParameterError::IllegalParameterName(value)
    }
}
