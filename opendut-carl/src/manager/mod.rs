pub mod peer_messaging_broker;
pub mod cluster_manager;
pub mod grpc;
pub mod peer_manager;
pub mod observer_messaging_broker;

#[cfg(test)]
mod testing {
    use crate::resource::manager::ResourceManagerRef;
    use opendut_types::cluster::{ClusterConfiguration, ClusterId, ClusterName};
    use opendut_types::peer::executor::ExecutorDescriptors;
    use opendut_types::peer::{PeerDescriptor, PeerId, PeerLocation, PeerName, PeerNetworkDescriptor};
    use opendut_types::topology::{DeviceDescription, DeviceDescriptor, DeviceId, DeviceName, Topology};
    use opendut_types::util::net::{NetworkInterfaceConfiguration, NetworkInterfaceDescriptor, NetworkInterfaceId, NetworkInterfaceName};
    use std::collections::HashSet;

    pub struct PeerFixture {
        pub id: PeerId,
        pub descriptor: PeerDescriptor,
        pub device_1: DeviceId,
        pub device_2: DeviceId,
    }
    impl PeerFixture {
        pub fn new() -> PeerFixture {
            let id = PeerId::random();
            let network_interface_1 = NetworkInterfaceId::random();
            let network_interface_2 = NetworkInterfaceId::random();
            let device_1 = DeviceId::random();
            let device_2 = DeviceId::random();

            let descriptor = PeerDescriptor {
                id,
                name: PeerName::try_from("PeerA").unwrap(),
                location: PeerLocation::try_from("Ulm").ok(),
                network: PeerNetworkDescriptor {
                    interfaces: vec![
                        NetworkInterfaceDescriptor {
                            id: network_interface_1,
                            name: NetworkInterfaceName::try_from("eth0").unwrap(),
                            configuration: NetworkInterfaceConfiguration::Ethernet,
                        },
                        NetworkInterfaceDescriptor {
                            id: network_interface_2,
                            name: NetworkInterfaceName::try_from("eth1").unwrap(),
                            configuration: NetworkInterfaceConfiguration::Ethernet,
                        },
                    ],
                    bridge_name: Some(NetworkInterfaceName::try_from("br-opendut-1").unwrap()),
                },
                topology: Topology {
                    devices: vec![
                        DeviceDescriptor {
                            id: device_1,
                            name: DeviceName::try_from("PeerA_Device_1").unwrap(),
                            description: DeviceDescription::try_from("Huii").ok(),
                            interface: network_interface_1,
                            tags: vec![],
                        },
                        DeviceDescriptor {
                            id: device_2,
                            name: DeviceName::try_from("PeerA_Device_2").unwrap(),
                            description: DeviceDescription::try_from("Huii").ok(),
                            interface: network_interface_2,
                            tags: vec![],
                        }
                    ]
                },
                executors: ExecutorDescriptors {
                    executors: vec![],
                }
            };
            PeerFixture {
                id,
                descriptor,
                device_1,
                device_2,
            }
        }
    }



    pub struct ClusterFixture {
        pub id: ClusterId,
        pub configuration: ClusterConfiguration,
        pub peer_a: PeerFixture,
        pub peer_b: PeerFixture,
    }

    impl ClusterFixture {
        pub async fn create(resource_manager: ResourceManagerRef) -> anyhow::Result<ClusterFixture> {
            let peer_a = PeerFixture::new();
            let peer_b = PeerFixture::new();

            resource_manager.insert(peer_a.id, peer_a.descriptor.clone()).await?;
            resource_manager.insert(peer_b.id, peer_b.descriptor.clone()).await?;

            let cluster_id = ClusterId::random();
            let cluster_configuration = ClusterConfiguration {
                id: cluster_id,
                name: ClusterName::try_from(format!("Cluster-{cluster_id}"))?,
                leader: peer_a.id,
                devices: HashSet::from([peer_a.device_1, peer_a.device_2, peer_b.device_1]),
            };
            resource_manager.insert(cluster_id, cluster_configuration.clone()).await?;

            Ok(ClusterFixture {
                id: cluster_id,
                configuration: cluster_configuration,
                peer_a,
                peer_b,
            })

        }
    }
}
