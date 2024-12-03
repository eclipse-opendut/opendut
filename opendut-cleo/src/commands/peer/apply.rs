use opendut_types::peer::{PeerDescriptor, PeerId, PeerLocation, PeerName, PeerNetworkDescriptor};
use opendut_types::peer::executor::{ExecutorDescriptor, ExecutorDescriptors, ExecutorId, ExecutorKind};
use opendut_types::peer::executor::container::{ContainerImage, Engine};
use opendut_types::specs::peer::{DeviceSpecificationV1, NetworkInterfaceDescriptorSpecificationV1, NetworkInterfaceKind, PeerDescriptorSpecificationV1, ExecutorSpecificationV1, SpecificationEngineKind, SpecificationExecutorKind};
use opendut_types::specs::SpecificationMetadata;
use opendut_types::topology::{DeviceDescription, DeviceDescriptor, DeviceId, DeviceName, DeviceTag, Topology};
use opendut_types::util::net::{CanSamplePoint, NetworkInterfaceConfiguration, NetworkInterfaceDescriptor, NetworkInterfaceId, NetworkInterfaceName};

pub fn convert_document_to_peer_descriptor(specification_metadata: SpecificationMetadata, peer: PeerDescriptorSpecificationV1) -> crate::Result<PeerDescriptor> {
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
    
    let topology = peer.topology.devices.into_iter()
        .map(convert_device_specification_to_descriptor)
        .collect::<Result<Vec<_>, _>>()?;

    let executors = peer.executors.into_iter()
        .map(convert_executor_specification_to_descriptor)
        .collect::<Result<Vec<_>, _>>()?;

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
            executors,
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

fn convert_executor_specification_to_descriptor(specification: ExecutorSpecificationV1) -> crate::Result<ExecutorDescriptor> {
    let kind = match specification.kind {
        SpecificationExecutorKind::Executable => ExecutorKind::Executable,
        SpecificationExecutorKind::Container => {
            match specification.parameters {
                Some(parameters) => {
                    let engine = match parameters.engine {
                        SpecificationEngineKind::Docker => Engine::Docker,
                        SpecificationEngineKind::Podman => Engine::Podman,
                    };

                    ExecutorKind::Container {
                        engine,
                        name: Default::default(),
                        image: ContainerImage::try_from("TestImage")
                            .map_err(|error| format!("Could not use the provided container image parameter for container executor <{}>:  {}", specification.id, error))?,
                        volumes: vec![],
                        devices: vec![],
                        envs: vec![],
                        ports: vec![],
                        command: Default::default(),
                        args: vec![],
                    }
                }
                None => Err(String::from("Parameters for the container executor were not provided."))?,
            }
        }
    };

    let executor_descriptor = ExecutorDescriptor {
        id: ExecutorId::from(specification.id),
        kind,
        results_url: None,
    };

    Ok(executor_descriptor)
}

#[cfg(test)]
mod tests {
    use super::*;
    use googletest::prelude::*;
    use opendut_types::peer::executor::{ExecutorDescriptor, ExecutorId, ExecutorKind};
    use opendut_types::peer::executor::container::{ContainerImage, Engine};
    use opendut_types::specs::peer::{DeviceSpecificationV1, ExecutorConfigurationSpecification, NetworkInterfaceConfigurationSpecification, NetworkInterfaceDescriptorSpecificationV1, NetworkInterfaceKind, TopologySpecificationV1, ExecutorSpecificationV1, NetworkDescriptorSpecificationV1, SpecificationEngineKind, SpecificationExecutorKind};
    use opendut_types::topology::{DeviceDescription, DeviceDescriptor, DeviceId, DeviceName, DeviceTag};
    use opendut_types::util::net::{NetworkInterfaceConfiguration, NetworkInterfaceDescriptor, NetworkInterfaceId, NetworkInterfaceName};

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

        let executors = peer.executors.executors.clone().into_iter()
            .map(convert_executor_descriptor_to_specification)
            .collect::<anyhow::Result<Vec<_>>>()?;
        
        let topology = get_topology_specification(peer.topology.devices[0].clone())?;
        let network = get_interface_specification(peer.clone(), interface_kind)?;
        
        let specification_peer = PeerDescriptorSpecificationV1 {
            location: peer.location.clone().map(|location| location.value()),
            network,
            topology,
            executors,
        };

        let result = convert_document_to_peer_descriptor(specification_metadata, specification_peer).unwrap();

        assert_that!(result, eq(&peer));

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
            executors: ExecutorDescriptors { executors: vec![ // TODO
                ExecutorDescriptor {
                    id: ExecutorId::random(),
                    kind: ExecutorKind::Container {
                        engine: Engine::Docker,
                        name: Default::default(), //ContainerName::Value(String::from("TestContainer")),
                        image: ContainerImage::try_from("TestImage")?,
                        volumes: vec![],
                        devices: vec![],
                        envs: vec![],
                        ports: vec![],
                        command: Default::default(),
                        args: vec![],
                    },
                    results_url: None,
                }
            ] }, // TODO
        })
    }
    
    fn get_topology_specification(device: DeviceDescriptor) -> anyhow::Result<TopologySpecificationV1> {
        let description = device.clone().description
            .map(|description| description.to_string());

        let tags = device.clone().tags.into_iter()
            .map(|tag| tag.to_string())
            .collect::<Vec<String>>();
        
        Ok(TopologySpecificationV1 {
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
            bridge_name: Default::default(), // TODO
        })
    }

    fn convert_executor_descriptor_to_specification(executor: ExecutorDescriptor) -> anyhow::Result<ExecutorSpecificationV1> {

        let executor_kind = match executor.kind {
            ExecutorKind::Executable => {
                SpecificationExecutorKind::Executable
            }
            ExecutorKind::Container { .. } => {
                SpecificationExecutorKind::Container
            }
        };

        let engine_kind = match executor.kind {
            ExecutorKind::Executable => unimplemented!("executable not implemented"),
            ExecutorKind::Container { engine, .. } => {
                match engine {
                    Engine::Docker => SpecificationEngineKind::Docker,
                    Engine::Podman => SpecificationEngineKind::Podman,
                }
            }
        };

        Ok(ExecutorSpecificationV1 {
            id: executor.id.uuid,
            kind: executor_kind,
            parameters: Some(ExecutorConfigurationSpecification {
                engine: engine_kind,
                // TODO
            })
        })
    }
}
