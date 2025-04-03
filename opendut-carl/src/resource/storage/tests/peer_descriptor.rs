use opendut_types::peer::executor::container::{ContainerCommand, ContainerCommandArgument, ContainerDevice, ContainerEnvironmentVariable, ContainerImage, ContainerName, ContainerPortSpec, ContainerVolume, Engine};
use opendut_types::peer::executor::{ExecutorDescriptor, ExecutorDescriptors, ExecutorId, ExecutorKind, ResultsUrl};
use opendut_types::peer::{PeerDescriptor, PeerId, PeerLocation, PeerName, PeerNetworkDescriptor};
use opendut_types::topology::{DeviceDescription, DeviceDescriptor, DeviceId, DeviceName, DeviceTag, Topology};
use opendut_types::util::net::{CanSamplePoint, NetworkInterfaceConfiguration, NetworkInterfaceDescriptor, NetworkInterfaceId, NetworkInterfaceName};

use crate::resource::manager::{ResourceManager, ResourceManagerRef};
use crate::resource::persistence;

#[tokio::test]
async fn should_persist_peer_descriptor_in_memory() -> anyhow::Result<()> {
    let resource_manager = ResourceManager::new_in_memory();
    should_persist_peer_descriptor_implementation(resource_manager).await
}

#[test_with::no_env(SKIP_DATABASE_CONTAINER_TESTS)]
#[tokio::test]
async fn should_persist_peer_descriptor_in_database() -> anyhow::Result<()> {
    let db = persistence::testing::spawn_and_connect_resource_manager().await?;
    should_persist_peer_descriptor_implementation(db.resource_manager).await
}

async fn should_persist_peer_descriptor_implementation(resource_manager: ResourceManagerRef) -> anyhow::Result<()> {

    let testee = peer_descriptor()?;

    let result = resource_manager.get::<PeerDescriptor>(testee.id).await?;
    assert!(result.is_none());
    let result = resource_manager.list::<PeerDescriptor>().await?;
    assert!(result.is_empty());

    resource_manager.insert(testee.id, testee.clone()).await?;

    let result = resource_manager.get::<PeerDescriptor>(testee.id).await?;
    assert_eq!(result, Some(testee.clone()));
    let result = resource_manager.list::<PeerDescriptor>().await?;
    assert_eq!(result.len(), 1);
    assert_eq!(result.values().next(), Some(&testee));


    let testee = {
        let mut testee = testee.clone();
        let removed_device = testee.topology.devices.remove(0);
        testee.network.interfaces.retain(|interface| interface.id != removed_device.interface);
        testee
    };
    resource_manager.insert(testee.id, testee.clone()).await?;


    let result = resource_manager.remove::<PeerDescriptor>(testee.id).await?;
    assert_eq!(result, Some(testee.clone()));

    let result = resource_manager.get::<PeerDescriptor>(testee.id).await?;
    assert!(result.is_none());
    let result = resource_manager.list::<PeerDescriptor>().await?;
    assert!(result.is_empty());

    let result = resource_manager.remove::<PeerDescriptor>(testee.id).await?;
    assert_eq!(result, None);

    Ok(())
}

pub fn peer_descriptor() -> anyhow::Result<PeerDescriptor> {
    let network_interface_id1 = NetworkInterfaceId::random();
    let network_interface_id2 = NetworkInterfaceId::random();

    Ok(PeerDescriptor {
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
