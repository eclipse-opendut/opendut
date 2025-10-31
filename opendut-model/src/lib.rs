use uuid::{Uuid, uuid};

pub mod cleo;
pub mod cluster;
pub mod lea;
pub mod peer;
pub mod proto;
pub mod resources;
#[cfg(feature = "specs")]
pub mod specs;
#[cfg(feature = "viper")]
pub mod viper;
pub mod topology;
pub mod util;
pub mod vpn;

pub trait ShortName {
    fn short_name(&self) -> &'static str;

    fn short_names_joined(elements: &[impl ShortName]) -> String {
        elements.iter()
            .map(|element| element.short_name())
            .collect::<Vec<_>>()
            .join(", ")
    }
}

/// UUID namespace for UUIDv5 generation
pub const OPENDUT_UUID_NAMESPACE: Uuid = uuid!("0de64908-7040-4efa-a3cc-b5e0840649e5");


pub mod format {
    use std::fmt;

    /// Formats an ID and a name in a legible manner.
    /// ```
    /// use opendut_model::format::format_id_with_name;
    ///
    /// assert_eq!(
    ///     format_id_with_name("123", "my_name"),
    ///     String::from("\"my_name\" <123>")
    /// );
    /// ```
    pub fn format_id_with_name(id: impl fmt::Display, name: impl fmt::Display) -> String {
        format!("\"{name}\" <{id}>")
    }


    /// Formats an ID and an optional name in a legible manner.
    /// ```
    /// use opendut_model::format::format_id_with_optional_name;
    ///
    /// // with name
    /// assert_eq!(
    ///     format_id_with_optional_name("123", &Some("my_name")),
    ///     String::from("\"my_name\" <123>")
    /// );
    ///
    /// // without name
    /// assert_eq!(
    ///     format_id_with_optional_name("123", &Option::<String>::None),
    ///     String::from("<123>")
    /// );
    /// ```
    pub fn format_id_with_optional_name(id: impl fmt::Display, name: &Option<impl fmt::Display>) -> String {
        let name = name.as_ref()
            .map(|name| format!("\"{name}\" "))
            .unwrap_or_default();

        format!("{name}<{id}>")
    }
}
