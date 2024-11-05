use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug)]
pub enum PeerDescriptorSpecification {
    V1(PeerDescriptorSpecificationV1)
}

#[derive(Debug, Deserialize)]
pub struct PeerDescriptorSpecificationV1 {
    #[serde(default)]
    pub location: Option<String>,
    pub network: PeerNetworkDescriptorSpecificationV1,
}

#[derive(Debug, Deserialize)]
pub struct PeerNetworkDescriptorSpecificationV1 {
    pub interfaces: Vec<NetworkInterfaceDescriptorSpecificationV1>,
    pub bridge_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct NetworkInterfaceDescriptorSpecificationV1 {
    pub id: Uuid,
    pub name: String,
    pub kind: NetworkInterfaceKind,
    pub parameters: Option<NetworkInterfaceConfigurationSpecification>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub enum NetworkInterfaceKind {
    Ethernet,
    Can,
}

#[derive(Debug, Deserialize)]
pub struct NetworkInterfaceConfigurationSpecification {
    pub bitrate: u32,
    pub sample_point: f32,
    pub fd: bool,
    pub data_bitrate: u32,
    pub data_sample_point: f32,
}
