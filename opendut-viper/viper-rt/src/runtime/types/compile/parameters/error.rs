#[derive(Debug)]
#[allow(dead_code)]
#[non_exhaustive]
pub enum ParameterError {
    IllegalParameterName(InvalidParameterNameError),
    IllegalTextParameterValue(InvalidTextParameterValueError),
    IllegalNumberParameterValue(InvalidNumberParameterValueError),
}

#[derive(Debug)]
pub struct InvalidTextParameterValueError {
    pub value: String,
    pub kind: InvalidTextParameterValueErrorKind,
}

impl InvalidTextParameterValueError {
    pub fn new_empty_parameter_value_error() -> Self {
        Self { value: String::new(), kind: InvalidTextParameterValueErrorKind::Empty }
    }
    
    pub fn new_too_long_parameter_value_error(value: impl Into<String>, expected: usize, actual: usize) -> Self {
        Self { value: value.into(), kind: InvalidTextParameterValueErrorKind::TooLong { expected, actual } }
    }

    pub fn new_invalid_type_parameter_value_error(value: impl Into<String>, expected: String, actual: String) -> Self {
        Self { value: value.into(), kind: InvalidTextParameterValueErrorKind::InvalidType { expected, actual}}
    }
}

#[derive(Debug)]
pub enum InvalidTextParameterValueErrorKind {
    Empty,
    TooLong { expected: usize, actual: usize },
    InvalidType { expected: String, actual: String },
}


#[derive(Debug)]
pub struct InvalidNumberParameterValueError {
    pub value: i64,
    pub kind: InvalidNumberParameterValueErrorKind,
}

impl InvalidNumberParameterValueError {
    pub fn new_too_small_parameter_value_error(value: i64, expected: usize, actual: usize) -> Self {
        Self { value, kind: InvalidNumberParameterValueErrorKind::TooSmall { expected, actual }}
    }

    pub fn new_too_big_parameter_value_error(value: i64, expected: usize, actual: usize) -> Self {
        Self { value, kind: InvalidNumberParameterValueErrorKind::TooBig { expected, actual }}
    }

    pub fn new_invalid_type_parameter_value_error(value: i64, expected: String, actual: String) -> Self {
        Self { value, kind: InvalidNumberParameterValueErrorKind::InvalidType { expected, actual }}
    }
}

#[derive(Debug)]
pub enum InvalidNumberParameterValueErrorKind {
    TooSmall { expected: usize, actual: usize },
    TooBig { expected: usize, actual: usize },
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
