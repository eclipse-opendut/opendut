mod error;

pub use error::MetadataError;

/// A `Metadata` is a part of a [`Compilation`].
///
/// [`Compilation`]: crate::compile::Compilation
///
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Metadata {
    pub display_name: Option<String>,
    pub description: Option<String>,
}
