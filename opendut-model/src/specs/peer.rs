use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug)]
pub enum PeerDescriptorSpecification {
    V1(PeerDescriptorSpecificationV1)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all="kebab-case")]
pub struct PeerDescriptorSpecificationV1 {
    #[serde(default)]
    pub location: Option<String>,
    pub network: NetworkDescriptorSpecificationV1,
    pub topology: TopologySpecificationV1,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all="kebab-case")]
pub struct NetworkDescriptorSpecificationV1 {
    pub interfaces: Vec<NetworkInterfaceDescriptorSpecificationV1>,
    pub bridge_name: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all="kebab-case")]
pub struct NetworkInterfaceDescriptorSpecificationV1 {
    pub id: Uuid,
    pub name: String,
    pub kind: NetworkInterfaceKind,
    pub parameters: Option<NetworkInterfaceConfigurationSpecification>,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all="kebab-case")]
pub enum NetworkInterfaceKind {
    Ethernet,
    Can,
    Vcan,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all="kebab-case")]
pub struct NetworkInterfaceConfigurationSpecification {
    pub bitrate_kbps: u32,
    pub sample_point: f32,
    pub fd: bool,
    pub data_bitrate_kbps: u32,
    pub data_sample_point: f32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all="kebab-case")]
pub struct TopologySpecificationV1 {
    pub devices: Vec<DeviceSpecificationV1>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all="kebab-case")]
pub struct DeviceSpecificationV1 {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub interface_id: Uuid,
    #[serde(default)]
    pub tags: Vec<String>,
}
