pub mod cluster_configuration;
pub mod cluster_deployment;
pub mod cluster_device;
pub mod device_descriptor;
pub mod device_tag;
pub mod network_interface_descriptor;
pub mod network_interface_kind;
pub mod peer_descriptor;

pub enum Filter<T> {
    By(T),
    Not,
}
