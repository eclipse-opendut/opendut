use std::fmt::{Display, Formatter};
use std::ops::Not;
use std::slice::Iter;
use std::vec::IntoIter;

mod error;

pub use error::{
    InvalidParameterNameError,
    InvalidParameterNameErrorKind,
    ParameterError,
};

/// The `ParameterDescriptors` is a container for a set [`ParameterDescriptor`]s defined in a test
/// suite.
///
/// A `ParameterDescriptors` is a part of a [`Compilation`] and can be retrieved from its
/// getter-functions or by splitting a Compilation into its parts.
///
/// [`Compilation`]: crate::compile::Compilation
/// 
#[derive(Clone, Debug, Default)]
pub struct ParameterDescriptors {
    parameters: Vec<ParameterDescriptor>
}

impl ParameterDescriptors {

    pub fn new() -> Self {
        Default::default()
    }

    pub fn push(&mut self, parameter: ParameterDescriptor) {
        self.parameters.push(parameter);
    }

    pub fn iter(&self) -> Iter<'_, ParameterDescriptor> {
        self.parameters.iter()
    }

    pub fn is_empty(&self) -> bool {
        self.parameters.is_empty()
    }

    pub fn len(&self) -> usize {
        self.parameters.len()
    }
}

impl IntoIterator for ParameterDescriptors {
    type Item = ParameterDescriptor;
    type IntoIter = IntoIter<ParameterDescriptor>;

    fn into_iter(self) -> IntoIter<ParameterDescriptor> {
        self.parameters.into_iter()
    }
}

/// A `ParameterName` is an identifier for a parameter.
///
/// # Examples
/// ```
/// use opendut_viper_rt::compile::ParameterName;
///
/// let parameter_name = ParameterName::try_from("awesome_parameter").expect("Valid parameter name");
/// ```
///
/// # Invariants
///
/// The `ParameterName` wraps a [`String`] to enforce invariants that ensure a valid name.
///
/// **Must not be empty:**
/// ```should_panic
/// # use opendut_viper_rt::compile::ParameterName;
/// #
/// ParameterName::try_from("").unwrap();
/// ```
///
/// **Must only contain legal characters:**
///
/// ```should_panic
/// # use opendut_viper_rt::compile::ParameterName;
/// ParameterName::try_from("Hello World").unwrap();
/// ```
/// ```should_panic
/// # use opendut_viper_rt::compile::ParameterName;
/// ParameterName::try_from("Hello%World").unwrap();
/// ```
/// See [`ALLOWED_CHARACTERS`] for all allowed characters.
///
/// [`ALLOWED_CHARACTERS`]: ParameterName::ALLOWED_CHARACTERS
///
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ParameterName {
    value: String
}

impl ParameterName {
    pub const ALLOWED_CHARACTERS: &'static str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_-";

    pub fn as_str(&self) -> &str {
        self.value.as_str()
    }
}

impl PartialEq<String> for ParameterName {
    fn eq(&self, other: &String) -> bool {
        self.value.eq(other)
    }
}

impl TryFrom<String> for ParameterName {
    type Error = InvalidParameterNameError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Err(InvalidParameterNameError::new_empty_parameter_name_error());
        }
        if let Some(char) = value.chars().find(|char| Self::ALLOWED_CHARACTERS.contains(*char).not()) {
            return Err(InvalidParameterNameError::new_illegal_parameter_name_character_error(value, char))
        }
        Ok(Self { value })
    }
}

impl TryFrom<&str> for ParameterName {
    type Error = InvalidParameterNameError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_from(value.to_owned())
    }
}

impl From<ParameterName> for String {
    fn from(value: ParameterName) -> Self {
        value.value
    }
}

impl From<&ParameterName> for String {
    fn from(value: &ParameterName) -> Self {
        Clone::clone(&value.value)
    }
}

impl From<&ParameterName> for ParameterName {
    fn from(value: &ParameterName) -> Self {
        Clone::clone(value)
    }
}

impl From<&ParameterDescriptor> for ParameterName {
    fn from(value: &ParameterDescriptor) -> Self {
        Clone::clone(value.name())
    }
}

impl Display for ParameterName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

/// A `ParameterDescriptor` describes a single parameter of a test suite.
#[derive(Clone, Debug, PartialEq)]
pub enum ParameterDescriptor {
    BooleanParameter {
        /// Primary identifier for the parameter
        name: ParameterName,
        info: ParameterInfo,
        default: Option<bool>,
    },
    NumberParameter {
        /// Primary identifier for the parameter
        name: ParameterName,
        info: ParameterInfo,
        default: Option<i64>,
        min: i64,
        max: i64,
    },
    TextParameter {
        /// Primary identifier for the parameter
        name: ParameterName,
        info: ParameterInfo,
        default: Option<String>,
        max: u32,
    }
}

impl ParameterDescriptor {

    pub(crate) const BOOLEAN_PARAMETER_VALUE_TYPE_NAME: &'static str = "boolean";
    pub(crate) const NUMBER_PARAMETER_VALUE_TYPE_NAME: &'static str = "number";
    pub(crate) const TEXT_PARAMETER_VALUE_TYPE_NAME: &'static str = "text";

    /// Primary identifier for the parameter
    pub fn name(&self) -> &ParameterName {
        match self {
            ParameterDescriptor::BooleanParameter { name, .. } => name,
            ParameterDescriptor::NumberParameter { name, .. } => name,
            ParameterDescriptor::TextParameter { name, .. } => name,
        }
    }

    pub fn has_default_value(&self) -> bool {
        match self {
            ParameterDescriptor::BooleanParameter { default, .. } => default.is_some(),
            ParameterDescriptor::NumberParameter { default, .. } => default.is_some(),
            ParameterDescriptor::TextParameter { default, .. } => default.is_some(),
        }
    }

    pub(crate) const fn value_type_name(&self) -> &'static str {
        match self {
            ParameterDescriptor::BooleanParameter { .. } => Self::BOOLEAN_PARAMETER_VALUE_TYPE_NAME,
            ParameterDescriptor::NumberParameter { .. } => Self::NUMBER_PARAMETER_VALUE_TYPE_NAME,
            ParameterDescriptor::TextParameter { .. } => Self::TEXT_PARAMETER_VALUE_TYPE_NAME,
        }
    }
}

/// The `ParameterInfo` provides additional parameter information for displaying to a user.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ParameterInfo {
    pub display_name: Option<String>,
    pub description: Option<String>,
}
