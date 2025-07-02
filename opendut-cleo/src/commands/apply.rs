use crate::commands::cluster_descriptor::apply::convert_document_to_cluster_descriptor;
use crate::commands::cluster_descriptor::create::create_cluster_descriptor;
use crate::commands::peer::apply::convert_document_to_peer_descriptor;
use crate::commands::peer::create::create_peer;
use crate::CreateOutputFormat;
use opendut_carl_api::carl::CarlClient;
use opendut_types::cluster::ClusterDescriptor;
use opendut_types::peer::PeerDescriptor;
use opendut_types::specs::cluster::ClusterDescriptorSpecification;
use opendut_types::specs::parse::json::JsonSpecificationDocument;
use opendut_types::specs::parse::yaml::YamlSpecificationFile;
use opendut_types::specs::peer::PeerDescriptorSpecification;
use opendut_types::specs::{Specification, SpecificationDocument};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(clap::Parser)]
///Create openDuT resource form file
pub struct ApplyCli {
    #[arg(value_parser=parse_source)]
    from: Source,
    ///Text, JSON or prettified JSON as output format
    #[arg(global=true, value_enum, short, long, default_value_t=CreateOutputFormat::Text)]
    output: CreateOutputFormat,
}

impl ApplyCli {
    pub async fn execute(self, carl: &mut CarlClient) -> crate::Result<()> {
        match &self.from {
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
                            self.create_resource_in_carl_from_model(model, carl).await?;
                        }
                        
                        Ok(())
                    }
                    Err(cause) => {
                        Err(format!("Failed to parse specification: {cause}"))
                    }
                }
            }
            Source::Inline(InlineSource::Json(json)) => {
                let json_document = JsonSpecificationDocument::try_from_json_str(json.as_str())
                    .map_err(|cause| format!("Error while parsing JSON:\n  {cause}"))?;

                let document = SpecificationDocument::try_from(json_document)
                    .map_err(|cause| format!("Error while converting JSON document to specification model:\n  {cause}"))?;

                let model = convert_document_to_model(document)?;

                self.create_resource_in_carl_from_model(model, carl).await?;

                Ok(())
            }
        }
    }

    async fn create_resource_in_carl_from_model(&self, model: ResourceModel, carl: &mut CarlClient) -> crate::Result<()> {
        match model {
            ResourceModel::PeerDescriptor(model) => {
                create_peer(model, carl, &self.output).await?;
            }
            ResourceModel::ClusterDescriptor(model) => {
                create_cluster_descriptor(model, carl, &self.output).await?;
            }
        }
        Ok(())
    }
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum Source {
    File(PathBuf),
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
    ClusterDescriptor(ClusterDescriptor),
}

fn convert_document_to_model(specification_document: SpecificationDocument) -> crate::Result<ResourceModel> {
    let result = match specification_document.spec {
        Specification::PeerDescriptorSpecification(PeerDescriptorSpecification::V1(peer)) => {
            let peer_id = specification_document.metadata.id;
            let peer_descriptor = convert_document_to_peer_descriptor(specification_document.metadata, peer)
                .map_err(|error| format!("Could not parse the provided specification for peer <{peer_id}>.\n  {error}"))?;
            ResourceModel::PeerDescriptor(peer_descriptor)
        }
        Specification::ClusterDescriptorSpecification(ClusterDescriptorSpecification::V1(cluster_descriptor)) => {
            let cluster_descriptor_id = specification_document.metadata.id;
            let cluster_descriptor = convert_document_to_cluster_descriptor(specification_document.metadata, cluster_descriptor)
                .map_err(|error| format!("Could not parse the provided specification for cluster descriptor <{cluster_descriptor_id}>.\n {error}"))?;
            ResourceModel::ClusterDescriptor(cluster_descriptor)
        }        
    };
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use googletest::prelude::*;
    use opendut_types::peer::executor::ExecutorDescriptors;
    use opendut_types::peer::{PeerDescriptor, PeerId, PeerName, PeerNetworkDescriptor};
    use opendut_types::specs::peer::{NetworkDescriptorSpecificationV1, NetworkInterfaceDescriptorSpecificationV1, NetworkInterfaceKind, PeerDescriptorSpecification, PeerDescriptorSpecificationV1, TopologySpecificationV1};
    use opendut_types::specs::{Specification, SpecificationDocument, SpecificationMetadata};
    use opendut_types::topology::Topology;
    use opendut_types::util::net::{NetworkInterfaceConfiguration, NetworkInterfaceDescriptor, NetworkInterfaceId, NetworkInterfaceName};

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
        
        let network = get_interface_specification(peer.clone(), interface_kind)?;
        
        let document = SpecificationDocument {
            version: String::from("v1"),
            metadata: SpecificationMetadata {
                id: peer.id.uuid,
                name: peer.name.value().to_owned(),
            },
            spec: Specification::PeerDescriptorSpecification(PeerDescriptorSpecification::V1(PeerDescriptorSpecificationV1 {
                location: None,
                network,
                topology: TopologySpecificationV1 { devices: vec![] },
                executors: vec![],
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
    
    fn generate_peer_descriptor() -> anyhow::Result<PeerDescriptor> {
        Ok(PeerDescriptor {
            id: PeerId::random(),
            name: PeerName::try_from("peer1")?,
            location: None,
            network: PeerNetworkDescriptor {
                interfaces: vec![
                    NetworkInterfaceDescriptor {
                        id: NetworkInterfaceId::random(),
                        name: NetworkInterfaceName::try_from("eth0")?,
                        configuration: NetworkInterfaceConfiguration::Ethernet,
                    }
                ],
                bridge_name: Some(NetworkInterfaceName::try_from("br-opendut")?),
            },
            topology: Topology { devices: vec![] },
            executors: ExecutorDescriptors { executors: vec![] },
        })
    }
    
    fn get_interface_specification(peer: PeerDescriptor, interface_kind: NetworkInterfaceKind) -> anyhow::Result<NetworkDescriptorSpecificationV1> {
        Ok(NetworkDescriptorSpecificationV1 {
            interfaces: vec![
                NetworkInterfaceDescriptorSpecificationV1 {
                    id: peer.network.interfaces[0].id.uuid,
                    name: peer.network.interfaces[0].name.to_string(),
                    kind: interface_kind,
                    parameters: None,
                }
            ],
            bridge_name: peer.network.bridge_name.map(|name| name.name()),
        })
    }
}
