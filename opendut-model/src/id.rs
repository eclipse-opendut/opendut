use std::fmt::{Debug, Formatter};
use std::any::Any;
use std::marker::PhantomData;


/// Create an ID newtype which wraps a UUID.
/// ```
///# use opendut_model::create_id_type;
///
/// create_id_type!(ExampleId);
///
/// let id = ExampleId::random();
/// ```
#[macro_export]
macro_rules! create_id_type {
    ($type_name:ident) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, ::serde::Serialize, ::serde::Deserialize)]
        #[serde(transparent)]
        pub struct $type_name {
            pub uuid: ::uuid::Uuid,
        }

        impl $type_name {
            #[allow(unused)]
            pub fn random() -> Self {
                Self { uuid: ::uuid::Uuid::new_v4() }
            }
        }

        impl ::std::fmt::Display for $type_name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                write!(f, "{}", self.uuid)
            }
        }

        impl From<::uuid::Uuid> for $type_name {
            fn from(uuid: ::uuid::Uuid) -> Self {
                Self { uuid }
            }
        }

        impl TryFrom<&str> for $type_name {
            type Error = $crate::id::IllegalId<$type_name>;

            fn try_from(value: &str) -> Result<Self, Self::Error> {
                ::uuid::Uuid::parse_str(value)
                    .map(|uuid| Self { uuid })
                    .map_err(|_| Self::Error {
                        value: String::from(value),
                        id_type: ::std::marker::PhantomData::<$type_name>,
                    })
            }
        }

        impl TryFrom<String> for $type_name {
            type Error = $crate::id::IllegalId<$type_name>;

            fn try_from(value: String) -> Result<Self, Self::Error> {
                Self::try_from(value.as_str())
            }
        }

        impl ::std::str::FromStr for $type_name {
            type Err = $crate::id::IllegalId<$type_name>;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                Self::try_from(value)
            }
        }
    };
}

#[derive(thiserror::Error, Clone)]
#[error("Illegal {}: {}", std::any::type_name::<T>(), value)]
pub struct IllegalId<T: Any> {
    pub value: String,
    pub id_type: PhantomData<T>,
}

impl<T: 'static> Debug for IllegalId<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(&format!("IllegalId<{}>", std::any::type_name::<T>()))
            .field("value", &self.value)
            .finish()
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    create_id_type!(ExampleId);

    #[test]
    fn should_debug_format_illegal_id_error() {

        let error = ExampleId::from_str("not a UUID")
            .err().unwrap();

        let actual = format!("{error:?}");
        let expected = r#"IllegalId<opendut_model::id::tests::ExampleId> { value: "not a UUID" }"#;

        assert_eq!(actual, expected);
    }
}
