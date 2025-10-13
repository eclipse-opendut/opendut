use uuid::{Uuid, uuid};

pub mod cleo;
pub mod cluster;
pub mod lea;
pub mod peer;
pub mod proto;
pub mod resources;
#[cfg(feature = "specs")]
pub mod specs;
#[cfg(feature = "test")]
pub mod test;
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
