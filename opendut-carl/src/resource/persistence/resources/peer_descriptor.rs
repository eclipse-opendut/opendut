use super::Persistable;
use crate::resource::api::id::ResourceId;
use crate::resource::persistence;
use crate::resource::persistence::error::PersistenceResult;
use crate::resource::persistence::{Db, Memory};
use opendut_types::peer::executor::ExecutorDescriptors;
use opendut_types::peer::{PeerDescriptor, PeerId, PeerLocation, PeerName, PeerNetworkDescriptor};
use opendut_types::topology::Topology;
use redb::TableDefinition;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

impl Persistable for PeerDescriptor {

    fn insert(self, peer_id: PeerId, _: &mut Memory, db: &Db) -> PersistenceResult<()> {
        let key = persistence::Key::from(ResourceId::<Self>::into_id(peer_id));

        let mut table = db.read_write_table(PEER_DESCRIPTOR_TABLE)?;

        let value = {
            let PeerDescriptor { id, name, location, network, topology, executors } = self;
            SerializablePeerDescriptor {
                id, name, location, network, topology, executors,
            }
        };
        let value = serde_json::to_string(&value).unwrap(); //TODO don't unwrap

        table.insert(&key, value).unwrap(); //TODO don't unwrap

        Ok(())
    }

    fn remove(peer_id: PeerId, _: &mut Memory, db: &Db) -> PersistenceResult<Option<Self>> {
        let key = persistence::Key::from(ResourceId::<Self>::into_id(peer_id));

        let mut table = db.read_write_table(PEER_DESCRIPTOR_TABLE)?;

        let value = table.remove(&key).unwrap() //TODO don't unwrap
            .map(|value| {
                let peer_descriptor = serde_json::from_str::<SerializablePeerDescriptor>(&value.value()).unwrap(); //TODO don't unwrap

                let SerializablePeerDescriptor { id, name, location, network, topology, executors } = peer_descriptor;
                PeerDescriptor { id, name, location, network, topology, executors }
            });

        Ok(value)
    }

    fn get(peer_id: PeerId, _: &Memory, db: &Db) -> PersistenceResult<Option<Self>> {
        let key = persistence::Key::from(ResourceId::<Self>::into_id(peer_id));

        let value = db.read_table(PEER_DESCRIPTOR_TABLE)?
            .and_then(|table| {
                table.get(&key).unwrap() //TODO don't unwrap
                    .map(|value| {
                        let peer_descriptor = serde_json::from_str::<SerializablePeerDescriptor>(&value.value()).unwrap(); //TODO don't unwrap

                        let SerializablePeerDescriptor { id, name, location, network, topology, executors } = peer_descriptor;
                        PeerDescriptor { id, name, location, network, topology, executors }
                    })
            });
        Ok(value)
    }

    fn list(_: &Memory, db: &Db) -> PersistenceResult<HashMap<Self::Id, Self>> {

        let values = db.read_table(PEER_DESCRIPTOR_TABLE)?
            .map(|table| {
                table.iter().unwrap() //TODO don't unwrap
                    .map(|value| {
                        let (key, value) = value.unwrap(); //TODO don't unwrap
                        let id = ResourceId::<Self>::from_id(key.value().id);

                        let peer_descriptor = serde_json::from_str::<SerializablePeerDescriptor>(&value.value()).unwrap(); //TODO don't unwrap
                        let peer_descriptor = {
                            let SerializablePeerDescriptor { id, name, location, network, topology, executors } = peer_descriptor;
                            PeerDescriptor { id, name, location, network, topology, executors }
                        };

                        (id, peer_descriptor)
                    })
                    .collect::<HashMap<_, _>>()
            })
            .unwrap_or_default();

        Ok(values)
    }
}

const PEER_DESCRIPTOR_TABLE: persistence::TableDefinition = TableDefinition::new("peer_descriptor");

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct SerializablePeerDescriptor { //TODO From-implementation //TODO version-field, if nothing exists natively in redb
    pub id: PeerId,
    pub name: PeerName,
    pub location: Option<PeerLocation>,
    pub network: PeerNetworkDescriptor,
    pub topology: Topology,
    pub executors: ExecutorDescriptors,
}


#[cfg(test)]
mod tests {
    use super::*;
    use opendut_types::peer::executor::container::{ContainerCommand, ContainerCommandArgument, ContainerDevice, ContainerEnvironmentVariable, ContainerImage, ContainerName, ContainerPortSpec, ContainerVolume, Engine};
    //TODO remove?
        use opendut_types::peer::executor::{ExecutorDescriptor, ExecutorId, ExecutorKind, ResultsUrl};
    use opendut_types::topology::{DeviceDescription, DeviceDescriptor, DeviceId, DeviceName, DeviceTag};
    use opendut_types::util::net::{CanSamplePoint, NetworkInterfaceConfiguration, NetworkInterfaceDescriptor, NetworkInterfaceId, NetworkInterfaceName};

    #[test]
    fn peer_descriptor_should_be_serializable() -> anyhow::Result<()> {
        let testee = peer_descriptor()?;
        let json = serde_json::to_string(&testee)?;

        let result = serde_json::from_str(&json)?;
        assert_eq!(testee, result);

        Ok(())
    }


    pub fn peer_descriptor() -> anyhow::Result<SerializablePeerDescriptor> {
        let network_interface_id1 = NetworkInterfaceId::random();
        let network_interface_id2 = NetworkInterfaceId::random();

        Ok(SerializablePeerDescriptor {
            id: PeerId::random(),
            name: PeerName::try_from("testee_name")?,
            location: Some(PeerLocation::try_from("testee_location")?),
            network: PeerNetworkDescriptor {
                interfaces: vec![
                    NetworkInterfaceDescriptor {
                        id: network_interface_id1,
                        name: NetworkInterfaceName::try_from("eth0")?,
                        configuration: NetworkInterfaceConfiguration::Ethernet,
                    },
                    NetworkInterfaceDescriptor {
                        id: network_interface_id2,
                        name: NetworkInterfaceName::try_from("can0")?,
                        configuration: NetworkInterfaceConfiguration::Can {
                            bitrate: 11111,
                            sample_point: CanSamplePoint::try_from(0.222)?,
                            fd: true,
                            data_bitrate: 33333,
                            data_sample_point: CanSamplePoint::try_from(0.444)?,
                        },
                    },
                ],
                bridge_name: Some(NetworkInterfaceName::try_from("br0")?),
            },
            topology: Topology {
                devices: vec![
                    DeviceDescriptor {
                        id: DeviceId::random(),
                        name: DeviceName::try_from("device1")?,
                        description: Some(DeviceDescription::try_from("device1-description")?),
                        interface: network_interface_id1,
                        tags: vec![
                            DeviceTag::try_from("tag1")?,
                            DeviceTag::try_from("tag2")?,
                        ],
                    },
                    DeviceDescriptor {
                        id: DeviceId::random(),
                        name: DeviceName::try_from("device2")?,
                        description: Some(DeviceDescription::try_from("device2-description")?),
                        interface: network_interface_id2,
                        tags: vec![
                            DeviceTag::try_from("tag2")?,
                            DeviceTag::try_from("tag3")?,
                        ],
                    },
                ],
            },
            executors: ExecutorDescriptors {
                executors: vec![
                    ExecutorDescriptor {
                        id: ExecutorId::random(),
                        kind: ExecutorKind::Container {
                            engine: Engine::Podman,
                            name: ContainerName::try_from("container-name")?,
                            image: ContainerImage::try_from("container-image")?,
                            volumes: vec![
                                ContainerVolume::try_from("container-volume")?,
                            ],
                            devices: vec![
                                ContainerDevice::try_from("container-device")?,
                            ],
                            envs: vec![
                                ContainerEnvironmentVariable::new("env-name", "env-value")?,
                            ],
                            ports: vec![
                                ContainerPortSpec::try_from("8080:8080")?,
                            ],
                            command: ContainerCommand::try_from("ls")?,
                            args: vec![
                                ContainerCommandArgument::try_from("-la")?,
                            ],
                        },
                        results_url: None,
                    },
                    ExecutorDescriptor {
                        id: ExecutorId::random(),
                        kind: ExecutorKind::Executable,
                        results_url: Some(ResultsUrl::try_from("https://example.com/")?),
                    },
                ]
            },
        })
    }
}
