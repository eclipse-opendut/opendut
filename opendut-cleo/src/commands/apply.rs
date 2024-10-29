use std::fs;
use std::path::PathBuf;
use opendut_carl_api::carl::CarlClient;
use opendut_types::peer::{PeerDescriptor, PeerId, PeerLocation, PeerName, PeerNetworkDescriptor};
use opendut_types::peer::executor::ExecutorDescriptors;
use opendut_types::specs::{Specification, SpecificationDocument, SpecificationMetadata};
use opendut_types::specs::parse::json::JsonSpecificationDocument;
use opendut_types::specs::parse::yaml::YamlSpecificationFile;
use opendut_types::specs::peer::{NetworkInterfaceDescriptorSpecificationV1, PeerDescriptorSpecification, PeerDescriptorSpecificationV1};
use opendut_types::util::net::{NetworkInterfaceConfiguration, NetworkInterfaceDescriptor, NetworkInterfaceId, NetworkInterfaceName};
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
                            }).collect::<Result<_, _>>()?;

                        let models = convert_documents_to_models(specification_documents)?;
                        
                        for model in models {
                            create_peer(model, carl, &self.output).await?;
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

fn convert_documents_to_models(documents: Vec<SpecificationDocument>) -> crate::Result<Vec<PeerDescriptor>> { //TODO return both PeerDescriptor and ClusterConfiguration
    documents.into_iter().map(|specification_document| {
        let result = match specification_document.spec {
            Specification::PeerDescriptorSpecification(PeerDescriptorSpecification::V1(peer)) => {
                let peer_id = specification_document.metadata.id;
                convert_document_to_peer_descriptor(specification_document.metadata, peer)
                    .map_err(|error| format!("Could not parse the provided specification for peer <{}>.\n  {}", peer_id, error))?
            }
            Specification::ClusterConfigurationSpecification(_) => todo!()
        };
        Ok(result)
    }).collect()
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

    let network_interfaces = convert_network_specification_to_descriptor(peer.network.interfaces)
        .map_err(|error| error.to_string())?;

    let descriptor: PeerDescriptor = PeerDescriptor {
        id,
        name,
        location,
        network: PeerNetworkDescriptor {
            interfaces: network_interfaces,
            bridge_name: Default::default(), // TODO
        },
        topology: Default::default(), // TODO
        executors: ExecutorDescriptors {
            executors: vec![], // TODO
        },
    };
    Ok(descriptor)
}

fn convert_network_specification_to_descriptor(network_interface_descriptor_specification_v1: Vec<NetworkInterfaceDescriptorSpecificationV1>) -> crate::Result<Vec<NetworkInterfaceDescriptor>> {

    let network_descriptors = network_interface_descriptor_specification_v1
        .into_iter()
        .map(|spec| {
            let name = NetworkInterfaceName::try_from(spec.name).map_err(|error| error.to_string()).unwrap();
            let configuration = match spec.configuration.to_lowercase().as_str() {
                "ethernet" => NetworkInterfaceConfiguration::Ethernet,
                "can" => NetworkInterfaceConfiguration::Ethernet, //TODO
                _ => NetworkInterfaceConfiguration::Ethernet,
            };

            NetworkInterfaceDescriptor {
                id: NetworkInterfaceId::from(spec.id),
                name,
                configuration,
            }
        }).collect();

    Ok(network_descriptors)
}

#[cfg(test)]
mod tests {
    use googletest::prelude::*;
    use opendut_types::peer::{PeerDescriptor, PeerId, PeerLocation, PeerName, PeerNetworkDescriptor};
    use opendut_types::peer::executor::ExecutorDescriptors;
    use opendut_types::specs::{Specification, SpecificationDocument, SpecificationMetadata};
    use opendut_types::specs::peer::{NetworkInterfaceDescriptorSpecificationV1, PeerDescriptorSpecification, PeerDescriptorSpecificationV1, PeerNetworkDescriptorSpecificationV1};
    use opendut_types::util::net::{NetworkInterfaceConfiguration, NetworkInterfaceDescriptor, NetworkInterfaceId, NetworkInterfaceName};
    use crate::commands::apply::{convert_documents_to_models, convert_document_to_peer_descriptor};

    #[test]
    fn should_convert_document_to_peer_descriptor() -> anyhow::Result<()> {
        let peer = generate_peer_descriptor()?;
        
        let specification_metadata = SpecificationMetadata {
            id: peer.id.uuid,
            name: peer.name.clone().value(),
        };

        let specification_peer = PeerDescriptorSpecificationV1 {
            location: peer.location.clone().map(|location| location.value()),
            network: PeerNetworkDescriptorSpecificationV1 { 
                interfaces: vec![
                    NetworkInterfaceDescriptorSpecificationV1 {
                        id: peer.network.interfaces[0].id.uuid,
                        name: peer.network.interfaces[0].name.to_string(),
                        configuration: peer.network.interfaces[0].configuration.to_string(),
                    }
                ],
                bridge_name: None 
            },
        };

        let result = convert_document_to_peer_descriptor(specification_metadata, specification_peer).unwrap();

        assert_that!(result, eq(&peer));

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
            topology: Default::default(), // TODO
            executors: ExecutorDescriptors { executors: vec![] }, // TODO
        })
    }

    #[test]
    fn should_convert_document_to_model() -> anyhow::Result<()> {
        let peer_list = generate_peer_list()?;
        
        let documents = vec![
            SpecificationDocument {
                version: String::from("v1"),
                metadata: SpecificationMetadata {
                    id: peer_list[0].id.uuid,
                    name: peer_list[0].name.clone().value(),
                },
                spec: Specification::PeerDescriptorSpecification(PeerDescriptorSpecification::V1(PeerDescriptorSpecificationV1 {
                    location: Some(String::from("Ulm")),
                    network: PeerNetworkDescriptorSpecificationV1 {
                        interfaces: vec![
                            NetworkInterfaceDescriptorSpecificationV1 {
                                id: peer_list[0].network.interfaces[0].id.uuid,
                                name: peer_list[0].network.interfaces[0].name.to_string(),
                                configuration: peer_list[0].network.interfaces[0].configuration.to_string(),
                            }
                        ],
                        bridge_name: Default::default(), // TODO
                    },
                }))
            },
            SpecificationDocument {
                version: String::from("v1"),
                metadata: SpecificationMetadata {
                    id: peer_list[1].id.uuid,
                    name: peer_list[1].name.clone().value(),
                },
                spec: Specification::PeerDescriptorSpecification(PeerDescriptorSpecification::V1(PeerDescriptorSpecificationV1 {
                    location: Some(String::from("Stuttgart")),
                     network: PeerNetworkDescriptorSpecificationV1 {
                        interfaces: vec![
                            NetworkInterfaceDescriptorSpecificationV1 {
                                id: peer_list[1].network.interfaces[0].id.uuid,
                                name: peer_list[1].network.interfaces[0].name.to_string(),
                                configuration: peer_list[1].network.interfaces[0].configuration.to_string(),
                            }
                        ],
                        bridge_name: Default::default(), // TODO
                    },
                }))
            }
        ];

        let result = convert_documents_to_models(documents).unwrap();

        assert_that!(result, eq(&peer_list));

        Ok(())

    }

    fn generate_peer_list() -> anyhow::Result<Vec<PeerDescriptor>> {
        Ok(
            vec![
                PeerDescriptor {
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
                    topology: Default::default(), //TODO
                    executors: ExecutorDescriptors { executors: vec![] }, //TODO
                },
                PeerDescriptor {
                    id: PeerId::random(),
                    name: PeerName::try_from("peer2")?,
                    location: Some(PeerLocation::try_from("Stuttgart")?),
                    network: PeerNetworkDescriptor {
                        interfaces: vec![
                            NetworkInterfaceDescriptor {
                                id: NetworkInterfaceId::random(),
                                name: NetworkInterfaceName::try_from("eth1")?,
                                configuration: NetworkInterfaceConfiguration::Ethernet,
                            }
                        ],
                        bridge_name: Default::default(), // TODO
                    },
                    topology: Default::default(), //TODO
                    executors: ExecutorDescriptors { executors: vec![] }, //TODO
                }
            ]
        )
    }
}
