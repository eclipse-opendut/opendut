mod error;

use crate::compile::{ParameterDescriptor, ParameterDescriptors};
use crate::runtime::types::compile::parameters::ParameterName;
use std::fmt::{Debug, Formatter};
use std::vec::IntoIter;

pub use error::{
    BindParameterError,
    IncompleteParameterBindingsError
};

/// The `ParameterBindings` maps a set of parameters to their values.
///
/// A `ParameterBindings` can be created from a [`ParameterDescriptors`] instance using the
/// [`ParameterBindings::from`] function. It is also possible to create an empty `ParameterBindings`
/// by calling the [`ParameterBindings::new`].
///
/// After creating a `ParameterBindings` the [`ParameterBindings::bind`] function can be used to
/// bind a value to a parameter by providing the parameter's [`ParameterName`] and a [`BindingValue`].
///
/// When all parameters have been bound, the [`ParameterBindings::complete`] function can be used
/// to validate that all parameters have been bound to values or have default values available. That
/// function also transitions the `ParameterBindings` from [`Incomplete`] to [`Complete`] state.
///
/// [`ParameterDescriptors`]: crate::compile::ParameterDescriptors
/// [`ParameterBindings::from`]: crate::run::ParameterBindings::from
/// [`ParameterBindings::new`]: crate::run::ParameterBindings::new
/// [`ParameterBindings::bind`]: crate::run::ParameterBindings::bind
/// [`ParameterBindings::complete`]: crate::run::ParameterBindings::complete
///
pub struct ParameterBindings<State> {
    bindings: Vec<ParameterBinding>,
    _phantom: std::marker::PhantomData<State>,
}

/// Marker for the [`ParameterBindings`] to indicate binding of parameters has not been completed.
pub enum Incomplete {} // Intentionally empty! Zero-Sized-Type that no one can create a value of it.

/// Marker for the [`ParameterBindings`] to indicate binding of parameters has been completed.
pub enum Complete {} // Intentionally empty! Zero-Sized-Type that no one can create a value of it.

impl <State> ParameterBindings<State> {

    pub fn new() -> Self {
        Self {
            bindings: Vec::new(),
            _phantom: Default::default(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.bindings.is_empty()
    }

    pub fn len(&self) -> usize {
        self.bindings.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &ParameterBinding> {
        self.bindings.iter()
    }
}

impl ParameterBindings<Incomplete> {

    /// Binds a value given as [`BindingValue`] to a parameter denoted by the specified [`ParameterName`]
    ///
    /// This method validates that a value of the correct type is provided and that the value is
    /// within its bounds (if applicable), otherwise a [`BindParameterError`] is returned.
    ///
    pub fn bind(&mut self, parameter: &ParameterName, value: BindingValue) -> Result<(), BindParameterError> {
        let binding = self.bindings.iter_mut()
            .find(|binding| binding.name() == parameter)
            .ok_or_else(|| BindParameterError::new_parameter_not_found_error(parameter))?;
        binding.bind(value)
    }

    /// Takes a closure to bind all parameters of the `ParameterBindings`.
    ///
    /// The closure may return a [`BindParameterError`] when it is not possible to return a valid
    /// [`BindingValue`].
    ///
    /// To uphold its invariants, this method also validates that each value is of the correct type
    /// and that each value is within its bounds (if applicable), otherwise a [`BindParameterError`]
    /// is returned.
    ///
    /// **Note:** If an error occurs, then the `ParameterBindings` will be left in a 'half-bound'
    /// state. Therefore, an invocation of the [`ParameterBindings::complete`] method will fail.
    ///
    pub fn bind_each<F>(&mut self, mut f: F) -> Result<(), BindParameterError>
    where
        F: FnMut(&ParameterDescriptor) -> Result<BindingValue, BindParameterError>
    {
        for binding in self.bindings.iter_mut() {
            let value = f(&binding.descriptor)?;
            binding.bind(value)?;
        }
        Ok(())
    }

    /// Transitions the `ParameterBindings` from [`Incomplete`] to [`Complete`] state.
    ///
    /// This method validates that all parameters have been bound to values or have default values
    /// available, otherwise an [`IncompleteParameterBindingsError`] is returned.
    ///
    pub fn complete(self) -> Result<ParameterBindings<Complete>, IncompleteParameterBindingsError> {
        let bindings = self.bindings;
        let missing_parameters = bindings.iter()
            .filter(|binding| !binding.is_bound() && !binding.has_default_value())
            .map(|binding| binding.name())
            .collect::<Vec<_>>();
        if missing_parameters.is_empty() {
            Ok(ParameterBindings {
                bindings,
                _phantom: Default::default(),
            })
        }
        else {
            Err(IncompleteParameterBindingsError::new(missing_parameters.into_iter().map(ToOwned::to_owned).collect()))
        }
    }
}

impl ParameterBindings<Complete> {

    /// Returns the [`BindingValue`] of the parameter with the specified [`ParameterName`].
    ///
    pub fn get_value(&self, parameter: &ParameterName) -> BindingValue {
        self.bindings.iter()
            .find(|binding| binding.name() == parameter)
            .and_then(|binding| Clone::clone(&binding.value))
            .or_else(|| self.get_default(parameter))
            .expect("Complete `ParameterBindings` must have a value for each parameter!")
    }

    /// Returns the default value of the parameter with the specified [`ParameterName`]. If no
    /// default value has been assigned to the parameter, [`None`] is returned.
    ///
    pub fn get_default(&self, parameter: &ParameterName) -> Option<BindingValue> {
        self.bindings.iter()
            .find(|binding| binding.name() == parameter)
            .and_then(|binding| match &binding.descriptor {
                ParameterDescriptor::BooleanParameter { default, .. } =>
                    default.map(BindingValue::BooleanValue),
                ParameterDescriptor::NumberParameter { default, .. } =>
                    default.map(BindingValue::NumberValue),
                ParameterDescriptor::TextParameter { default, .. } =>
                    default.as_ref().map(|value| BindingValue::TextValue(Clone::clone(value))),
            })
    }
}

impl <State> Default for ParameterBindings<State> {
    fn default() -> Self {
        Self::new()
    }
}

impl <State> Clone for ParameterBindings<State> {
    fn clone(&self) -> Self {
        Self {
            bindings: Clone::clone(&self.bindings),
            _phantom: Default::default(),
        }
    }
}

impl <State> IntoIterator for ParameterBindings<State> {
    type Item = ParameterBinding;
    type IntoIter = IntoIter<ParameterBinding>;
    fn into_iter(self) -> Self::IntoIter {
        self.bindings.into_iter()
    }
}

impl From<ParameterDescriptors> for ParameterBindings<Incomplete> {
    fn from(descriptors: ParameterDescriptors) -> Self {
        Self {
            bindings: descriptors.into_iter()
                .map(ParameterBinding::new)
                .collect(),
            _phantom: Default::default(),
        }
    }
}

/// A single binding of a parameter to its value.
#[derive(Clone, Debug)]
pub struct ParameterBinding {
    pub descriptor: ParameterDescriptor,
    pub value: Option<BindingValue>,
}

impl ParameterBinding {

    pub fn new(descriptor: ParameterDescriptor) -> Self {
        Self {
            descriptor,
            value: None,
        }
    }

    pub fn name(&self) -> &ParameterName {
        self.descriptor.name()
    }

    pub fn is_bound(&self) -> bool {
        self.value.is_some()
    }

    pub fn has_default_value(&self) -> bool {
        self.descriptor.has_default_value()
    }

    fn bind(&mut self, value: BindingValue) -> Result<(), BindParameterError> {
        match (&self.descriptor, &value) {
            (ParameterDescriptor::BooleanParameter { .. }, BindingValue::BooleanValue(..)) => self.value = Some(value),
            (ParameterDescriptor::NumberParameter { min, max, .. }, BindingValue::NumberValue(actual_value)) => {
                if *actual_value < *min || *actual_value > *max {
                    return Err(BindParameterError::new_number_value_out_of_range_error(self.descriptor.name(), *actual_value, *min, *max))
                }
                self.value = Some(value)
            }
            (ParameterDescriptor::TextParameter { max, .. }, BindingValue::TextValue(actual_value)) => {
                if actual_value.len() > *max as usize {
                    return Err(BindParameterError::new_text_value_out_of_range_error(self.descriptor.name(), actual_value, *max));
                }
                self.value = Some(value)
            }
            _ => return Err(BindParameterError::new_type_mismatch_error(self.descriptor.name(), self.descriptor.value_type_name(), value.value_type_name()))
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum BindingValue {
    BooleanValue(bool),
    NumberValue(i64),
    TextValue(String),
}

impl BindingValue {

    pub(crate) const fn value_type_name(&self) -> &'static str {
        match self {
            BindingValue::BooleanValue { .. } => ParameterDescriptor::BOOLEAN_PARAMETER_VALUE_TYPE_NAME,
            BindingValue::NumberValue { .. } => ParameterDescriptor::NUMBER_PARAMETER_VALUE_TYPE_NAME,
            BindingValue::TextValue { .. } => ParameterDescriptor::TEXT_PARAMETER_VALUE_TYPE_NAME,
        }
    }
}

impl <State> Debug for ParameterBindings<State> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ParameterBindings")
            .field("bindings", &self.bindings)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compile::ParameterInfo;
    use googletest::prelude::*;

    #[test]
    fn test_from_empty_descriptors() -> Result<()> {

        let bindings = ParameterBindings::from(ParameterDescriptors::new());

        assert_that!(bindings.is_empty(), eq(true));
        assert_that!(bindings.len(), eq(0));

        Ok(())
    }

    #[test]
    fn test_from_descriptors() -> Result<()> {

        let mut descriptors = ParameterDescriptors::new();

        descriptors.push(ParameterDescriptor::BooleanParameter {
            key: String::new(),
            name: ParameterName::try_from("some_boolean")?,
            info: ParameterInfo::default(),
            default: None,
        });

        descriptors.push(ParameterDescriptor::NumberParameter {
            key: String::new(),
            name: ParameterName::try_from("some_number")?,
            info: ParameterInfo::default(),
            default: None,
            min: 1,
            max: 10
        });

        descriptors.push(ParameterDescriptor::TextParameter {
            key: String::new(),
            name: ParameterName::try_from("some_text")?,
            info: ParameterInfo::default(),
            default: None,
            max: u32::MAX,
        });

        let bindings = ParameterBindings::from(descriptors);

        assert_that!(bindings.is_empty(), eq(false));
        assert_that!(bindings.len(), eq(3));

        Ok(())
    }

    #[test]
    fn test_bind_bool_successfully() -> Result<()> {

        let parameter = ParameterName::try_from("some_boolean")?;

        let mut descriptors = ParameterDescriptors::new();

        descriptors.push(ParameterDescriptor::BooleanParameter {
            key: String::new(),
            name: Clone::clone(&parameter),
            info: ParameterInfo::default(),
            default: None,
        });

        let mut bindings = ParameterBindings::from(descriptors);

        bindings.bind(&parameter, BindingValue::BooleanValue(true))?;

        let bindings = bindings.complete()?;

        assert_that!(bindings.get_value(&parameter), eq(&BindingValue::BooleanValue(true)));

        Ok(())
    }

    #[test]
    fn test_bind_number_successfully() -> Result<()> {

        let parameter = ParameterName::try_from("some_number")?;

        let mut descriptors = ParameterDescriptors::new();

        descriptors.push(ParameterDescriptor::NumberParameter {
            key: String::new(),
            name: Clone::clone(&parameter),
            info: ParameterInfo::default(),
            default: None,
            min: 1,
            max: 10
        });

        let mut bindings = ParameterBindings::from(descriptors);

        bindings.bind(&parameter, BindingValue::NumberValue(5))?;

        let bindings = bindings.complete()?;

        assert_that!(bindings.get_value(&parameter), eq(&BindingValue::NumberValue(5)));

        Ok(())
    }

    #[test]
    fn test_bind_text_successfully() -> Result<()> {

        let parameter = ParameterName::try_from("some_text")?;

        let mut descriptors = ParameterDescriptors::new();

        descriptors.push(ParameterDescriptor::TextParameter {
            key: String::new(),
            name: Clone::clone(&parameter),
            info: ParameterInfo::default(),
            default: None,
            max: u32::MAX,
        });

        let mut bindings = ParameterBindings::from(descriptors);

        bindings.bind(&parameter, BindingValue::TextValue(String::from("Hello World")))?;

        let bindings = bindings.complete()?;

        assert_that!(bindings.get_value(&parameter), eq(&BindingValue::TextValue(String::from("Hello World"))));

        Ok(())
    }

    #[test]
    fn test_bind_fails_when_parameter_does_not_exist() -> Result<()> {

        let mut descriptors = ParameterDescriptors::new();

        descriptors.push(ParameterDescriptor::TextParameter {
            key: String::new(),
            name: ParameterName::try_from("foo")?,
            info: ParameterInfo::default(),
            default: None,
            max: u32::MAX,
        });

        let mut bindings = ParameterBindings::from(descriptors);

        let result = bindings.bind(&ParameterName::try_from("bar")?, BindingValue::TextValue(String::from("Hello World")));

        assert_that!(result, err(eq(&BindParameterError::ParameterNotFound(ParameterName::try_from("bar")?))));

        Ok(())
    }

    #[test]
    fn test_bind_fails_when_type_does_not_match() -> Result<()> {

        let parameter = ParameterName::try_from("some_text")?;

        let mut descriptors = ParameterDescriptors::new();

        descriptors.push(ParameterDescriptor::TextParameter {
            key: String::new(),
            name: Clone::clone(&parameter),
            info: ParameterInfo::default(),
            default: None,
            max: u32::MAX,
        });

        let mut bindings = ParameterBindings::from(descriptors);

        let result = bindings.bind(&parameter, BindingValue::NumberValue(8121));

        assert_that!(result, err(eq(&BindParameterError::TypeMismatch {
            parameter_name: ParameterName::try_from("some_text")?,
            expected_type: String::from("text"),
            actual_type: String::from("number"),
        })));

        Ok(())
    }

    #[test]
    fn test_bind_fails_when_number_value_is_out_of_range() -> Result<()> {

        let parameter = ParameterName::try_from("my_number")?;

        let mut descriptors = ParameterDescriptors::new();

        descriptors.push(ParameterDescriptor::NumberParameter {
            key: String::new(),
            name: Clone::clone(&parameter),
            info: ParameterInfo::default(),
            default: None,
            min: 1,
            max: 3,
        });

        let mut bindings = ParameterBindings::from(descriptors);

        let result = bindings.bind(&parameter, BindingValue::NumberValue(0));

        assert_that!(result, err(eq(&BindParameterError::NumberValueOutOfRange {
            parameter_name: ParameterName::try_from("my_number")?,
            value: 0,
            min: 1,
            max: 3,
        })));

        let result = bindings.bind(&parameter, BindingValue::NumberValue(4));

        assert_that!(result, err(eq(&BindParameterError::NumberValueOutOfRange {
            parameter_name: ParameterName::try_from("my_number")?,
            value: 4,
            min: 1,
            max: 3,
        })));

        Ok(())
    }

    #[test]
    fn test_bind_fails_when_text_value_is_out_of_range() -> Result<()> {

        let parameter = ParameterName::try_from("really_short_text")?;

        let mut descriptors = ParameterDescriptors::new();

        descriptors.push(ParameterDescriptor::TextParameter {
            key: String::new(),
            name: Clone::clone(&parameter),
            info: ParameterInfo::default(),
            default: None,
            max: 3,
        });

        let mut bindings = ParameterBindings::from(descriptors);

        let result = bindings.bind(&parameter, BindingValue::TextValue(String::from("Hello World")));

        assert_that!(result, err(eq(&BindParameterError::TextValueOutOfRange {
            parameter_name: ParameterName::try_from("really_short_text")?,
            value: String::from("Hello World"),
            max: 3,
        })));

        Ok(())
    }

    #[test]
    fn test_bind_each() -> Result<()> {

        let parameter_a = ParameterName::try_from("a")?;
        let parameter_b = ParameterName::try_from("b")?;

        let mut descriptors = ParameterDescriptors::new();

        descriptors.push(ParameterDescriptor::BooleanParameter {
            key: String::new(),
            name: Clone::clone(&parameter_a),
            info: ParameterInfo::default(),
            default: None,
        });

        descriptors.push(ParameterDescriptor::TextParameter {
            key: String::new(),
            name: Clone::clone(&parameter_b),
            info: ParameterInfo::default(),
            default: None,
            max: u32::MAX,
        });

        let mut bindings = ParameterBindings::from(descriptors);

        bindings.bind_each(|parameter| {
            if parameter.name() == &parameter_a {
                Ok(BindingValue::BooleanValue(true))
            }
            else if parameter.name() == &parameter_b {
                Ok(BindingValue::TextValue(String::from("Hello World")))
            }
            else {
                Err(BindParameterError::new_parameter_not_found_error(parameter))
            }
        })?;

        let result = bindings.complete();

        assert_that!(result, ok(anything()));

        Ok(())
    }

    #[test]
    fn test_complete_succeeds_when_all_parameters_have_a_value_bound() -> Result<()> {

        let parameter = ParameterName::try_from("really_short_text")?;

        let mut descriptors = ParameterDescriptors::new();

        descriptors.push(ParameterDescriptor::BooleanParameter {
            key: String::new(),
            name: Clone::clone(&parameter),
            info: ParameterInfo::default(),
            default: None,
        });

        let mut bindings = ParameterBindings::from(descriptors);

        bindings.bind(&parameter, BindingValue::BooleanValue(true))?;

        let result = bindings.complete();

        assert_that!(result, ok(anything()));

        Ok(())
    }

    #[test]
    fn test_complete_succeeds_when_there_are_parameters_with_no_value_bound_but_with_default_value() -> Result<()> {

        let parameter = ParameterName::try_from("really_short_text")?;

        let mut descriptors = ParameterDescriptors::new();

        descriptors.push(ParameterDescriptor::BooleanParameter {
            key: String::new(),
            name: Clone::clone(&parameter),
            info: ParameterInfo::default(),
            default: Some(false),
        });

        let bindings = ParameterBindings::from(descriptors);

        let result = bindings.complete();

        assert_that!(result, ok(anything()));

        Ok(())
    }

    #[test]
    fn test_complete_fails_when_there_are_parameters_with_no_value_bound() -> Result<()> {

        let parameter = ParameterName::try_from("really_short_text")?;

        let mut descriptors = ParameterDescriptors::new();

        descriptors.push(ParameterDescriptor::BooleanParameter {
            key: String::new(),
            name: Clone::clone(&parameter),
            info: ParameterInfo::default(),
            default: None,
        });

        let bindings = ParameterBindings::from(descriptors);

        let result = bindings.complete();

        assert_that!(result, err(eq(&IncompleteParameterBindingsError::new(vec![parameter]))));

        Ok(())
    }

    #[test]
    fn test_get_value_returns_default_value() -> Result<()> {

        let parameter = ParameterName::try_from("some_text")?;

        let mut descriptors = ParameterDescriptors::new();

        descriptors.push(ParameterDescriptor::TextParameter {
            key: String::new(),
            name: Clone::clone(&parameter),
            default: Some(String::from("Hello World")),
            info: ParameterInfo::default(),
            max: u32::MAX,
        });

        let bindings = ParameterBindings::from(descriptors).complete()?;

        assert_that!(bindings.get_value(&parameter), eq(&BindingValue::TextValue(String::from("Hello World"))));

        Ok(())
    }

    #[test]
    fn test_into_inter() -> Result<()> {

        let text_parameter = ParameterName::try_from("some_text")?;
        let number_parameter = ParameterName::try_from("some_number")?;

        let mut descriptors = ParameterDescriptors::new();

        descriptors.push(ParameterDescriptor::TextParameter {
            key: String::new(),
            name: Clone::clone(&text_parameter),
            default: None,
            info: ParameterInfo::default(),
            max: u32::MAX,
        });

        descriptors.push(ParameterDescriptor::NumberParameter {
            key: String::new(),
            name: Clone::clone(&number_parameter),
            default: None,
            info: ParameterInfo::default(),
            min: -5,
            max: 5,
        });

        let bindings = ParameterBindings::from(descriptors);

        assert_that!(bindings.into_iter().collect::<Vec<_>>(), len(eq(2)));

        Ok(())
    }
}
