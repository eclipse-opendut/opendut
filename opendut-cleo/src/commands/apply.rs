use std::fs;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use opendut_carl_api::carl::CarlClient;
use opendut_types::cluster::{ClusterConfiguration, ClusterId, ClusterName};
use opendut_types::peer::{PeerDescriptor, PeerId, PeerLocation, PeerName, PeerNetworkDescriptor};
use opendut_types::peer::executor::ExecutorDescriptors;
use opendut_types::specs::{Specification, SpecificationDocument, SpecificationMetadata};
use opendut_types::specs::cluster::{ClusterConfigurationSpecification, ClusterConfigurationSpecificationV1};
use opendut_types::specs::parse::json::JsonSpecificationDocument;
use opendut_types::specs::parse::yaml::YamlSpecificationFile;
use opendut_types::specs::peer::{DeviceSpecificationV1, NetworkInterfaceDescriptorSpecificationV1, NetworkInterfaceKind, PeerDescriptorSpecification, PeerDescriptorSpecificationV1};
use opendut_types::topology::{DeviceDescription, DeviceDescriptor, DeviceId, DeviceName, DeviceTag, Topology};
use opendut_types::util::net::{CanSamplePoint, NetworkInterfaceConfiguration, NetworkInterfaceDescriptor, NetworkInterfaceId, NetworkInterfaceName};
use crate::commands::peer::create::create_peer;
use crate::CreateOutputFormat;

#[derive(clap::Parser)]
///Create openDuT resource form file
pub struct ApplyCli {
    #[arg(short, long, value_parser=parse_source)]
    from: Source,
    ///Text, JSON or prettified JSON as output format
    #[arg(global=true, value_enum, short, long, default_value_t=CreateOutputFormat::Text)]
    output: CreateOutputFormat,
}

impl ApplyCli {
    pub async fn execute(self, carl: &mut CarlClient) -> crate::Result<()> {
        match self.from {
            Source::File(path) => {
                let content = fs::read_to_string(path).unwrap();
                match YamlSpecificationFile::try_from_yaml_str(&content) {
                    Ok(file) => {
                        let specification_documents = file.documents
                            .into_iter()
                            .map(|yaml_specification| {
                                SpecificationDocument::try_from(yaml_specification)
                                    .map_err(|error| error.to_string())
                            }).collect::<Result<Vec<_>, _>>()?;

                        let models = specification_documents.into_iter()
                            .map(convert_document_to_model)
                            .collect::<Result<Vec<_>, _>>()?;
                        
                        for model in models {
                            match model {
                                ResourceModel::PeerDescriptor(model) => {
                                    create_peer(model, carl, &self.output).await?
                                }
                                ResourceModel::ClusterConfiguration(model) => {
                                    carl.cluster.store_cluster_configuration(model).await
                                        .map_err(|err| format!("Could not store cluster configuration. Make sure the application is running. Error: {}", err))?;
                                }
                            };
                        }
                        
                        Ok(())
                        // TODO store_cluster_configuration
                    }
                    Err(cause) => {
                        Err(format!("Failed to parse specification: {cause}"))
                    }
                }
            }
            Source::Inline(InlineSource::Json(json)) => {
                match JsonSpecificationDocument::try_from_json_str(json.as_str()) {
                    Ok(_) => {}
                    Err(_) => {}
                }
                todo!("unsupported source")
            }
            Source::Url(_) => {
                todo!("unsupported source")
            }
        }
    }
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum Source {
    File(PathBuf),
    Url(String),
    Inline(InlineSource)
}

#[derive(Clone, Debug)]
pub enum InlineSource {
    Json(String)
}

#[derive(thiserror::Error, Debug)]
#[error("'{arg}' cannot be used as source. {details}")]
struct SourceParsingError {
    arg: String,
    details: String,
}

fn parse_source(arg: &str) -> Result<Source, SourceParsingError> {

    if arg.starts_with('{') && arg.ends_with('}') {
        Ok(Source::Inline(InlineSource::Json(arg.to_owned())))
    }
    else {
        let path = PathBuf::from(arg);
        if !path.exists() {
            Err(SourceParsingError { arg: String::from(arg), details: String::from("Path does not exist.") })?;
        }
        if path.is_file() || path.is_dir() {
            Ok(Source::File(path))
        }
        else {
            Err(SourceParsingError { arg: String::from(arg), details: String::from("Path designates neither a file nor a directory.") })
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
enum ResourceModel {
    PeerDescriptor(PeerDescriptor),
    ClusterConfiguration(ClusterConfiguration),
}

fn convert_document_to_model(specification_document: SpecificationDocument) -> crate::Result<ResourceModel> {
    let result = match specification_document.spec {
        Specification::PeerDescriptorSpecification(PeerDescriptorSpecification::V1(peer)) => {
            let peer_id = specification_document.metadata.id;
            let peer_descriptor = convert_document_to_peer_descriptor(specification_document.metadata, peer)
                .map_err(|error| format!("Could not parse the provided specification for peer <{}>.\n  {}", peer_id, error))?;
            ResourceModel::PeerDescriptor(peer_descriptor)
        }
        Specification::ClusterConfigurationSpecification(ClusterConfigurationSpecification::V1(cluster_configuration)) => {
            let cluster_configuration_id = specification_document.metadata.id;
            let cluster_configuration = convert_document_to_cluster_configuration(specification_document.metadata, cluster_configuration)
                .map_err(|error| format!("Could not parse the provided specification for cluster configuration <{}>.\n {}", cluster_configuration_id, error))?;
            ResourceModel::ClusterConfiguration(cluster_configuration)
        }        
    };
    Ok(result)
}

fn convert_document_to_peer_descriptor(specification_metadata: SpecificationMetadata, peer: PeerDescriptorSpecificationV1) -> crate::Result<PeerDescriptor> {
    let SpecificationMetadata { id, name } = specification_metadata;

    let id = PeerId::from(id);

    let name = PeerName::try_from(name)
        .map_err(|error| error.to_string())?;

    let location = peer.location
        .map(PeerLocation::try_from)
        .transpose()
        .map_err(|error| error.to_string())?;
    
    let network_interfaces = peer.network.interfaces.into_iter()
        .map(convert_network_specification_to_descriptor)
        .collect::<Result<Vec<_>, _>>()?;
    
    let topology = peer.topology
        .map(|topology| topology.devices.into_iter()
            .map(convert_device_specification_to_descriptor)
            .collect::<Result<Vec<_>, _>>())
        .transpose()?
        .unwrap_or_default();
    
    let descriptor: PeerDescriptor = PeerDescriptor {
        id,
        name,
        location,
        network: PeerNetworkDescriptor {
            interfaces: network_interfaces,
            bridge_name: Default::default(), // TODO
        },
        topology: Topology {
            devices: topology
        },
        executors: ExecutorDescriptors {
            executors: vec![], // TODO
        },
    };
    Ok(descriptor)
}

fn convert_network_specification_to_descriptor(specification: NetworkInterfaceDescriptorSpecificationV1) -> crate::Result<NetworkInterfaceDescriptor> {
    
    let name = NetworkInterfaceName::try_from(specification.name.clone())
        .map_err(|error| error.to_string())?;
    
    let configuration = match specification.kind {
        NetworkInterfaceKind::Ethernet => NetworkInterfaceConfiguration::Ethernet,
        NetworkInterfaceKind::Can => {
            match specification.parameters {
                Some(parameters) => {
                    NetworkInterfaceConfiguration::Can {
                        bitrate: parameters.bitrate_hz,
                        sample_point: CanSamplePoint::try_from(parameters.sample_point)
                            .map_err(|error| format!("Could not use the provided sample point parameter for network interface <{}>:  {}", specification.id, error))?,
                        fd: parameters.fd,
                        data_bitrate: parameters.data_bitrate_hz,
                        data_sample_point: CanSamplePoint::try_from(parameters.data_sample_point)
                            .map_err(|error| format!("Could not use the provided data sample point parameter for network interface <{}>:  {}", specification.id, error))?,
                    }
                }
                None => Err(String::from("Parameters for the can interface were not provided."))?,
            }
        }
    };

    let network_descriptor = NetworkInterfaceDescriptor {
        id: NetworkInterfaceId::from(specification.id),
        name,
        configuration,
    };

    Ok(network_descriptor)
}

fn convert_device_specification_to_descriptor(specification: DeviceSpecificationV1) -> crate::Result<DeviceDescriptor> {
    let tags = specification.tags.into_iter().map(|tag| 
        DeviceTag::try_from(tag)
            .map_err(|error| format!("Could not apply the provided device tags for device: <{}>:  {}", specification.id, error))
    ).collect::<Result<Vec<_>, _>>()?;
    
    let description = specification.description
        .map(DeviceDescription::try_from)
        .transpose()
        .map_err(|error| error.to_string())?;
    
    let device_descriptor = DeviceDescriptor {
        id: DeviceId::from(specification.id),
        name: DeviceName::try_from(specification.name)
            .map_err(|error| format!("Could not apply the provided device name for device: <{}>:  {}", specification.id, error))?,
        description,
        interface: NetworkInterfaceId::from(specification.interface_id),
        tags,
    };
    
    Ok(device_descriptor)
}

fn convert_document_to_cluster_configuration(specification_metadata: SpecificationMetadata, cluster: ClusterConfigurationSpecificationV1) -> crate::Result<ClusterConfiguration>  {
    let SpecificationMetadata { id, name } = specification_metadata;
    
    let id = ClusterId::from(id);

    let name = ClusterName::try_from(name)
        .map_err(|error| error.to_string())?;
    
    let leader = cluster.leader_id; 
    
    let configuration = ClusterConfiguration {
        id,
        name,
        leader: PeerId::from(leader),
        devices: Default::default(),
    };
    
    Ok(configuration)
}


#[cfg(test)]
mod tests {
    use super::*;
    use googletest::prelude::*;
    use opendut_types::cluster::{ClusterConfiguration, ClusterId, ClusterName};
    use opendut_types::peer::{PeerDescriptor, PeerId, PeerLocation, PeerName, PeerNetworkDescriptor};
    use opendut_types::peer::executor::ExecutorDescriptors;
    use opendut_types::specs::{Specification, SpecificationDocument, SpecificationMetadata};
    use opendut_types::specs::cluster::ClusterConfigurationSpecificationV1;
    use opendut_types::specs::peer::{DeviceSpecificationV1, NetworkInterfaceConfigurationSpecification, NetworkInterfaceDescriptorSpecificationV1, NetworkInterfaceKind, PeerDescriptorSpecification, PeerDescriptorSpecificationV1, PeerDeviceSpecificationV1, PeerNetworkDescriptorSpecificationV1};
    use opendut_types::util::net::{CanSamplePoint, NetworkInterfaceConfiguration, NetworkInterfaceDescriptor, NetworkInterfaceId, NetworkInterfaceName};

    #[test]
    fn should_convert_document_to_model() -> anyhow::Result<()> {
        let peer = generate_peer_descriptor()?;
        
        let interface_kind = match peer.network.interfaces[0].configuration {
            NetworkInterfaceConfiguration::Ethernet => {
                NetworkInterfaceKind::Ethernet
            }
            NetworkInterfaceConfiguration::Can { .. } => {
                NetworkInterfaceKind::Can
            }
        };
        
        let topology = get_topology_specification(peer.topology.devices[0].clone())?;
        let network = get_interface_specification(peer.clone(), interface_kind)?;
        
        let document = SpecificationDocument {
            version: String::from("v1"),
            metadata: SpecificationMetadata {
                id: peer.id.uuid,
                name: peer.name.clone().value(),
            },
            spec: Specification::PeerDescriptorSpecification(PeerDescriptorSpecification::V1(PeerDescriptorSpecificationV1 {
                location: Some(String::from("Ulm")),
                network,
                topology: Some(topology),
            }))
        };
        
        let model = convert_document_to_model(document).unwrap();
        let result =
            if let ResourceModel::PeerDescriptor(model) = model {
                model
            }
            else {
                panic!("Specification is not a peer.")
            };
        
        assert_that!(result, eq(&peer));

        Ok(())
    }

    #[test]
    fn should_convert_document_to_peer_descriptor() -> anyhow::Result<()> {
        let peer = generate_peer_descriptor()?;
        
        let specification_metadata = SpecificationMetadata {
            id: peer.id.uuid,
            name: peer.name.clone().value(),
        };
        
        let interface_kind = match peer.network.interfaces[0].configuration {
            NetworkInterfaceConfiguration::Ethernet => {
                NetworkInterfaceKind::Ethernet
            }
            NetworkInterfaceConfiguration::Can { .. } => {
                NetworkInterfaceKind::Can
            }
        };
        
        let topology = get_topology_specification(peer.topology.devices[0].clone())?;
        let network = get_interface_specification(peer.clone(), interface_kind)?;
        
        let specification_peer = PeerDescriptorSpecificationV1 {
            location: peer.location.clone().map(|location| location.value()),
            network,
            topology: Some(topology),
        };

        let result = convert_document_to_peer_descriptor(specification_metadata, specification_peer).unwrap();

        assert_that!(result, eq(&peer));

        Ok(())
    }
    
    #[test]
    fn should_convert_document_to_cluster_configuration() -> anyhow::Result<()> {
        let cluster_configuration = ClusterConfiguration {
            id: ClusterId::random(),
            name: ClusterName::try_from("FirstCluster")?,
            leader: PeerId::random(),
            devices: Default::default(),
        };
        
        let specification_meta_data = SpecificationMetadata {
            id: cluster_configuration.id.clone().0,
            name: cluster_configuration.name.clone().value(),
        };
        
        let document = ClusterConfigurationSpecificationV1 {
            leader_id: cluster_configuration.leader.uuid,
        };
        
        let result = convert_document_to_cluster_configuration(specification_meta_data, document).unwrap();

        assert_that!(result, eq(&cluster_configuration));


        Ok(())      
    }
    
    #[test]
    fn should_convert_ethernet_network_specification_to_descriptor() -> anyhow::Result<()> {
        let specification =  NetworkInterfaceDescriptorSpecificationV1 {
            id: NetworkInterfaceId::random().uuid,
            name: "eth0".to_string(),
            kind: NetworkInterfaceKind::Ethernet,
            parameters: None,
        };
        
        let configuration = match specification.kind {
            NetworkInterfaceKind::Ethernet => NetworkInterfaceConfiguration::Ethernet,
            NetworkInterfaceKind::Can => NetworkInterfaceConfiguration::Can {
                bitrate: 0,
                sample_point: CanSamplePoint::try_from(0.7)?,
                fd: false,
                data_bitrate: 0,
                data_sample_point: CanSamplePoint::try_from(0.7)?,
            }
        };
        
        let descriptor = NetworkInterfaceDescriptor {
            id: NetworkInterfaceId::from(specification.id),
            name: NetworkInterfaceName::try_from(specification.name.clone())?,
            configuration,
        };
        
        let result = convert_network_specification_to_descriptor(specification).unwrap();
        
        assert_that!(result, eq(&descriptor));
        Ok(())
    } 

    #[test]
    fn should_convert_can_network_specification_to_descriptor() -> anyhow::Result<()> {
        let specification = NetworkInterfaceDescriptorSpecificationV1 {
            id: NetworkInterfaceId::random().uuid,
            name: "can0".to_string(),
            kind: NetworkInterfaceKind::Can,
            parameters: Some(
                NetworkInterfaceConfigurationSpecification {
                    bitrate_hz: 500000,
                    sample_point: 0.7,
                    fd: true,
                    data_bitrate_hz: 200000,
                    data_sample_point: 0.7,
                }
            ),
        };
        
        let descriptor = NetworkInterfaceDescriptor {
            id: NetworkInterfaceId::from(specification.id),
            name: NetworkInterfaceName::try_from(specification.name.clone())?,
            configuration: NetworkInterfaceConfiguration::Can {
                bitrate: 500000,
                sample_point: CanSamplePoint::try_from(0.7)?,
                fd: true,
                data_bitrate: 200000,
                data_sample_point: CanSamplePoint::try_from(0.7)?,
            },
        };

        let result = convert_network_specification_to_descriptor(specification).unwrap();
        
        assert_that!(result, eq(&descriptor));
        Ok(())
    }
    
    #[test]
    fn should_convert_device_specification_to_descriptor() -> anyhow::Result<()> {
        let specification = DeviceSpecificationV1 {
            id: DeviceId::random().0,
            name: "device".to_string(),
            description: None,
            interface_id: NetworkInterfaceId::random().uuid,
            tags: vec![
                String::from("new")
            ],
        };

        let tags = specification.tags.iter()
            .map(|tag_str| DeviceTag::try_from(tag_str.as_str()).expect("Failed to convert tag"))
            .collect();
        
        let descriptor = DeviceDescriptor {
            id: DeviceId::from(specification.id),
            name: DeviceName::try_from(specification.name.clone())?,
            description: specification.description.clone().map(|d| DeviceDescription::try_from(d).unwrap()),
            interface: NetworkInterfaceId::from(specification.interface_id),
            tags,
        };
        
        let result = convert_device_specification_to_descriptor(specification).unwrap();
        
        assert_that!(result, eq(&descriptor));
        Ok(())
    }

    fn generate_peer_descriptor() -> anyhow::Result<PeerDescriptor> {
        Ok(PeerDescriptor {
            id: PeerId::random(),
            name: PeerName::try_from("peer1")?,
            location: Some(PeerLocation::try_from("Ulm")?),
            network: PeerNetworkDescriptor {
                interfaces: vec![
                    NetworkInterfaceDescriptor {
                        id: NetworkInterfaceId::random(),
                        name: NetworkInterfaceName::try_from("eth0")?,
                        configuration: NetworkInterfaceConfiguration::Ethernet,
                    }
                ],
                bridge_name: Default::default(), // TODO
            },
            topology: Topology {
                devices: vec![
                    DeviceDescriptor {
                        id: DeviceId::random(),
                        name: DeviceName::try_from("device1")?,
                        description: Some(DeviceDescription::try_from("This a short device description.")?),
                        interface: NetworkInterfaceId::random(),
                        tags: vec![
                            DeviceTag::try_from("first-device")?
                        ],
                    }
                ],
            },
            executors: ExecutorDescriptors { executors: vec![] }, // TODO
        })
    }
    
    fn get_topology_specification(device: DeviceDescriptor) -> anyhow::Result<PeerDeviceSpecificationV1> {
        let description = device.clone().description
            .map(|description| description.to_string());

        let tags = device.clone().tags.into_iter()
            .map(|tag| tag.to_string())
            .collect::<Vec<String>>();
        
        Ok(PeerDeviceSpecificationV1 {
            devices: vec![
                DeviceSpecificationV1 {
                    id: device.id.0,
                    name: device.name.to_string(),
                    description,
                    interface_id: device.interface.uuid,
                    tags,
                }
            ]
        })
    }    
    
    fn get_interface_specification(peer: PeerDescriptor, interface_kind: NetworkInterfaceKind) -> anyhow::Result<PeerNetworkDescriptorSpecificationV1> {
        Ok(PeerNetworkDescriptorSpecificationV1 {
            interfaces: vec![
                NetworkInterfaceDescriptorSpecificationV1 {
                    id: peer.network.interfaces[0].id.uuid,
                    name: peer.network.interfaces[0].name.to_string(),
                    kind: interface_kind,
                    parameters: None,
                }
            ],
            bridge_name: Default::default(), // TODO
        })
    }
}
