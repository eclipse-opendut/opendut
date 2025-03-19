pub mod cluster;
pub mod peer;
pub mod topology;
pub mod util;
pub mod vpn;
pub mod cleo;

use std::marker::PhantomData;

#[derive(thiserror::Error, Debug, Eq, PartialEq)]
#[error("Could not convert from `{from}` to `{to}`: {details}")]
pub struct ConversionError {
    from: &'static str,
    to: &'static str,
    details: String,
}

impl ConversionError {
    pub fn new<From, To>(details: impl Into<String>) -> Self {
        Self {
            from: std::any::type_name::<From>(),
            to: std::any::type_name::<To>(),
            details: details.into(),
        }
    }
}

pub type ConversionResult<T> = Result<T, ConversionError>;


pub struct ConversionErrorBuilder<From, To> {
    _from: PhantomData<From>,
    _to: PhantomData<To>,
}

#[allow(clippy::new_ret_no_self)]
impl<From, To> ConversionErrorBuilder<From, To> {
    pub fn message(details: impl Into<String>) -> ConversionError {
        ConversionError::new::<From, To>(details)
    }
    pub fn field_not_set(field: impl Into<String>) -> ConversionError {
        let details = format!("Field '{}' not set", field.into());
        ConversionError::new::<From, To>(details)
    }
}


#[macro_export]
macro_rules! conversion {
    (
        type Model = $Model:ty;
        type Proto = $Proto:ty;

        $from_function_definition:item
        $try_from_function_definition:item
    ) => {
        impl From<$Model> for $Proto {
            fn from(value: $Model) -> Self {
                type Model = $Model;
                type Proto = $Proto;

                $from_function_definition

                from(value) //calls templated function definition
            }
        }

        impl TryFrom<$Proto> for $Model {
            type Error = ConversionError;

            fn try_from(value: $Proto) -> $crate::proto::ConversionResult<Self> {
                #[allow(unused)]
                type ErrorBuilder = ConversionErrorBuilder<$Proto, $Model>;
                type Model = $Model;
                type Proto = $Proto;

                #[allow(unused)]
                macro_rules! extract {
                    ($field:ident) => {
                        $field
                            .ok_or(ErrorBuilder::field_not_set(stringify!($field)))
                    };
                    ($value:ident.$field:ident) => {
                        $value.$field
                            .ok_or(ErrorBuilder::field_not_set(stringify!($field)))
                    };
                    ($value:ident.$field1:ident.$field2:ident) => {
                        $value.$field1
                            .ok_or(ErrorBuilder::field_not_set(stringify!($field1)))?
                            .$field2
                            .ok_or(ErrorBuilder::field_not_set(stringify!($field2)))
                    };
                }

                $try_from_function_definition

                try_from(value) //calls templated function definition
            }
        }
    }
}
pub(crate) use conversion; //makes macro available in module tree like a normal element
