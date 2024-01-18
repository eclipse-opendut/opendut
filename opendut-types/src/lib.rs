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
