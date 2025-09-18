#[derive(Debug)]
#[allow(dead_code)]
#[non_exhaustive]
pub enum ParameterError {
    IllegalParameterName(InvalidParameterNameError),
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
