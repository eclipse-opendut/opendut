use std::fs;
use std::path::PathBuf;

use opendut_carl_api::carl::CarlClient;
use opendut_types::peer::{PeerDescriptor, PeerId, PeerLocation, PeerName};
use opendut_types::peer::executor::ExecutorDescriptors;
use opendut_types::specs::{PeerDescriptorSpecification, PeerDescriptorSpecificationV1, Specification, SpecificationDocument, SpecificationMetadata};
use opendut_types::specs::yaml::{YamlSpecificationFile};

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
    pub async fn execute(self, _carl: &mut CarlClient) -> crate::Result<()> {
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
                        
                        
                        todo!("store_peer_descriptor and store_cluster_configuration")
                    }
                    Err(cause) => {
                        Err(format!("Failed to parse specification: {cause}"))
                    }
                }
            }
            Source::Inline(InlineSource::Json(json)) => {
                match Specification::from_json_str(json.as_str()) {
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

    let descriptor: PeerDescriptor = PeerDescriptor {
        id,
        name,
        location,
        network: Default::default(), // TODO
        topology: Default::default(), // TODO
        executors: ExecutorDescriptors {
            executors: vec![], // TODO
        },
    };
    Ok(descriptor)
}

#[cfg(test)]
mod tests {
    use googletest::prelude::*;
    use opendut_types::peer::{PeerDescriptor, PeerId, PeerLocation, PeerName};
    use opendut_types::peer::executor::ExecutorDescriptors;
    use opendut_types::specs::{PeerDescriptorSpecification, PeerDescriptorSpecificationV1, Specification, SpecificationDocument, SpecificationMetadata};
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
            network: Default::default(), //TODO
            topology: Default::default(), //TODO
            executors: ExecutorDescriptors { executors: vec![] }, //TODO
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
                    location: Some(String::from("Ulm"))
                }))
            },
            SpecificationDocument {
                version: String::from("v1"),
                metadata: SpecificationMetadata {
                    id: peer_list[1].id.uuid,
                    name: peer_list[1].name.clone().value(),
                },
                spec: Specification::PeerDescriptorSpecification(PeerDescriptorSpecification::V1(PeerDescriptorSpecificationV1 {
                    location: Some(String::from("Stuttgart"))
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
                    network: Default::default(), //TODO
                    topology: Default::default(), //TODO
                    executors: ExecutorDescriptors { executors: vec![] }, //TODO
                },
                PeerDescriptor {
                    id: PeerId::random(),
                    name: PeerName::try_from("peer2")?,
                    location: Some(PeerLocation::try_from("Stuttgart")?),
                    network: Default::default(), //TODO
                    topology: Default::default(), //TODO
                    executors: ExecutorDescriptors { executors: vec![] }, //TODO
                }
            ]
        )
    }
}
