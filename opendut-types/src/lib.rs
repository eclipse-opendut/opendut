use uuid::{Uuid, uuid};

pub mod cluster;
pub mod peer;
pub mod proto;
pub mod topology;
pub mod vpn;
pub mod util;

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
