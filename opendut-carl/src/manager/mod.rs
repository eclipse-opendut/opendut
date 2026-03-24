pub mod peer_messaging_broker;
pub mod cluster_manager;
pub mod grpc;
pub mod peer_manager;
pub mod observer_messaging_broker;
#[cfg(feature = "viper")]
pub mod test_manager;

#[cfg(test)]
pub(crate) mod testing {
    use crate::resource::manager::ResourceManagerRef;
    use opendut_model::cluster::{ClusterDescriptor, ClusterId, ClusterName};
    use opendut_model::peer::executor::ExecutorDescriptors;
    use opendut_model::peer::{PeerDescriptor, PeerId, PeerLocation, PeerName, PeerNetworkDescriptor};
    use opendut_model::topology::{DeviceDescription, DeviceDescriptor, DeviceId, DeviceName, Topology};
    use opendut_model::util::net::{NetworkInterfaceConfiguration, NetworkInterfaceDescriptor, NetworkInterfaceId, NetworkInterfaceName};
    use std::collections::HashSet;
    use opendut_util::pem::{read_pem_from_buffer, Pem};

    pub fn get_cert() -> Pem {
        let pems = read_pem_from_buffer(CERTIFICATE_AUTHORITY_STRING, "insecure-development-ca.pem").expect("Not a valid certificate!");
        pems.first().cloned().expect("Could not extract root CA").clone()
    }

    const CERTIFICATE_AUTHORITY_STRING: &str = include_str!("../../../resources/development/tls/insecure-development-ca.pem");


    pub struct PeerFixture {
        pub id: PeerId,
        pub descriptor: PeerDescriptor,
        pub device_1: DeviceId,
        pub device_2: DeviceId,
    }
    impl PeerFixture {
        pub fn new() -> Self {
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
            Self {
                id,
                descriptor,
                device_1,
                device_2,
            }
        }
    }


    pub struct ClusterFixture {
        pub id: ClusterId,
        pub descriptor: ClusterDescriptor,
        pub peer_a: PeerFixture,
        pub peer_b: PeerFixture,
    }
    impl ClusterFixture {
        pub async fn create(resource_manager: ResourceManagerRef) -> anyhow::Result<Self> {
            let peer_a = PeerFixture::new();
            let peer_b = PeerFixture::new();

            resource_manager.insert(peer_a.id, peer_a.descriptor.clone()).await?;
            resource_manager.insert(peer_b.id, peer_b.descriptor.clone()).await?;

            let cluster_id = ClusterId::random();
            let cluster_descriptor = ClusterDescriptor {
                id: cluster_id,
                name: ClusterName::try_from(format!("Cluster-{cluster_id}"))?,
                leader: peer_a.id,
                devices: HashSet::from([peer_a.device_1, peer_a.device_2, peer_b.device_1]),
            };
            resource_manager.insert(cluster_id, cluster_descriptor.clone()).await?;

            Ok(Self {
                id: cluster_id,
                descriptor: cluster_descriptor,
                peer_a,
                peer_b,
            })
        }
    }


    #[cfg(feature = "viper")]
    pub use viper::*;
    #[cfg(feature = "viper")]
    mod viper {
        use super::*;
        use opendut_model::viper::{ViperRunDeployment, ViperRunId, ViperSourceDescriptor, ViperSourceId, ViperSourceName, ViperTestRunDescriptor, ViperTestId, ViperTestName, ViperTestParameterKey, ViperTestParameterValue};
        use url::Url;
        use std::collections::HashMap;

        pub struct ViperSourceFixture {
            pub id: ViperSourceId,
            pub descriptor: ViperSourceDescriptor,
        }
        impl ViperSourceFixture {
            pub async fn create(resource_manager: ResourceManagerRef) -> anyhow::Result<Self> {
                let source_id = ViperSourceId::random();
                let source_descriptor = ViperSourceDescriptor {
                    id: source_id,
                    name: ViperSourceName::try_from(format!("ViperSource-{source_id}"))?,
                    url: Url::parse("http://localhost")?,
                };

                resource_manager.insert(source_id, source_descriptor.clone()).await?;

                Ok(Self {
                    id: source_id,
                    descriptor: source_descriptor,
                })
            }
        }

        pub struct ViperTestFixture {
            pub id: ViperTestId,
            pub descriptor: ViperTestRunDescriptor,
        }
        impl ViperTestFixture {
            pub async fn create(resource_manager: ResourceManagerRef) -> anyhow::Result<Self> {
                let source = ViperSourceFixture::create(resource_manager.clone()).await?;
                let cluster = ClusterFixture::create(resource_manager.clone()).await?;

                let test_id = ViperTestId::random();

                let parameters = {
                    let mut parameters: HashMap<ViperTestParameterKey, ViperTestParameterValue> = HashMap::new();
                    parameters.insert(
                        ViperTestParameterKey { inner: String::from("parameter-key") },
                        ViperTestParameterValue::Boolean(true),
                    );
                    parameters
                };

                let test_descriptor = ViperTestRunDescriptor {
                    id: test_id,
                    name: ViperTestName::try_from(format!("ViperTest-{test_id}"))?,
                    source: source.id,
                    cluster: cluster.id,
                    parameters,
                };

                resource_manager.insert(test_id, test_descriptor.clone()).await?;

                Ok(Self {
                    id: test_id,
                    descriptor: test_descriptor,
                })
            }
        }

        pub struct ViperRunDeploymentFixture {
            pub id: ViperRunId,
            pub deployment: ViperRunDeployment
        }

        impl ViperRunDeploymentFixture {
            pub async fn create(resource_manager: ResourceManagerRef) -> anyhow::Result<Self> {
                let test = ViperTestFixture::create(resource_manager.clone()).await?;

                let run_id = ViperRunId::random();

                let deployment = ViperRunDeployment {
                    run_id,
                    test_id: test.id,
                };

                resource_manager.insert(run_id, deployment.clone()).await?;

                Ok(Self {
                    id: run_id,
                    deployment,
                })
            }
        }
    }
}
