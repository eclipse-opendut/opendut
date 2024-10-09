use crate::testing::carl_client::TestCarlClient;
use crate::testing::util;
use googletest::prelude::*;
use opendut_types::cluster::{ClusterAssignment, ClusterConfiguration, ClusterDeployment, ClusterId, ClusterName, PeerClusterAssignment};
use opendut_types::peer::configuration::{OldPeerConfiguration, Parameter, ParameterTarget, PeerConfiguration};
use opendut_types::peer::ethernet::EthernetBridge;
use opendut_types::peer::executor::ExecutorDescriptors;
use opendut_types::peer::{PeerDescriptor, PeerId, PeerName, PeerNetworkDescriptor};
use opendut_types::topology::{DeviceDescriptor, DeviceId, DeviceName, Topology};
use opendut_types::util::net::{NetworkInterfaceConfiguration, NetworkInterfaceDescriptor, NetworkInterfaceId, NetworkInterfaceName};
use opendut_types::util::Port;
use std::collections::HashSet;
use std::net::IpAddr;
use std::str::FromStr;

#[test_log::test(
    tokio::test(flavor = "multi_thread")
)]
async fn carl_should_send_peer_configurations_in_happy_flow() -> anyhow::Result<()> {
    let fixture = Fixture::new();

    let carl_port = util::spawn_carl()?;

    let carl_client = TestCarlClient::connect(carl_port).await?;

    let peer_a = store_peer_descriptor(&carl_client).await?;

    let mut receiver_a = util::spawn_edgar_with_peer_configuration_receiver(peer_a.id, carl_port).await?;
    carl_client.await_peer_up(peer_a.id).await?;
    {
        let (peer_configuration_a, old_peer_configuration_a) = receiver_a.receive_peer_configuration().await?;
        assert_eq!(peer_configuration_a, fixture.empty_peer_configuration);
        assert_eq!(old_peer_configuration_a, fixture.empty_old_peer_configuration);
        receiver_a.expect_no_peer_configuration().await;
    }

    let peer_b = store_peer_descriptor(&carl_client).await?;

    let mut receiver_b = util::spawn_edgar_with_peer_configuration_receiver(peer_b.id, carl_port).await?;
    carl_client.await_peer_up(peer_b.id).await?;
    {
        let (peer_configuration_b, old_peer_configuration_b) = receiver_b.receive_peer_configuration().await?;
        assert_eq!(peer_configuration_b, fixture.empty_peer_configuration);
        assert_eq!(old_peer_configuration_b, fixture.empty_old_peer_configuration);
        receiver_b.expect_no_peer_configuration().await;
    }

    let cluster_leader = peer_a.id;
    let cluster_devices = peer_a.topology.devices.iter().chain(peer_b.topology.devices.iter());
    let cluster = store_cluster_configuration(cluster_leader, cluster_devices, &carl_client).await?;

    store_cluster_deployment(cluster.id, &carl_client).await?;

    {
        let validate_peer_configuration = |peer_configuration: PeerConfiguration| {
            assert_that!(peer_configuration, matches_pattern!(PeerConfiguration {
                executors: empty(),
                ethernet_bridges: contains(
                    matches_pattern!(Parameter {
                        id: anything(),
                        dependencies: empty(),
                        target: eq(&ParameterTarget::Present),
                        value: eq(&EthernetBridge {
                            name: NetworkInterfaceName::try_from("br-opendut")?,
                        }),
                    })
                ),
            }));
            Ok::<_, anyhow::Error>(())
        };
        let validate_old_peer_configuration = |old_peer_configuration: OldPeerConfiguration| {
            assert_that!(old_peer_configuration, matches_pattern!(OldPeerConfiguration {
                cluster_assignment: some(matches_pattern!(ClusterAssignment {
                    id: anything(),
                    leader: eq(&cluster_leader),
                    assignments: unordered_elements_are!(
                        matches_pattern!(PeerClusterAssignment {
                            peer_id: eq(&peer_a.id),
                            vpn_address: eq(&IpAddr::from_str("127.0.0.1")?),
                            can_server_port: any!(eq(&Port(10000)), eq(&Port(10001))),
                            device_interfaces: eq(&peer_a.network.interfaces),
                        }),
                        matches_pattern!(PeerClusterAssignment {
                            peer_id: eq(&peer_b.id),
                            vpn_address: eq(&IpAddr::from_str("127.0.0.1")?),
                            can_server_port: any!(eq(&Port(10000)), eq(&Port(10001))),
                            device_interfaces: eq(&peer_b.network.interfaces),
                        }),
                    ),
                }))
            }));
            Ok::<_, anyhow::Error>(())
        };

        let (peer_configuration_a, old_peer_configuration_a) = receiver_a.receive_peer_configuration().await?;
        validate_peer_configuration(peer_configuration_a)?;
        validate_old_peer_configuration(old_peer_configuration_a)?;

        let (peer_configuration_b, old_peer_configuration_b) = receiver_b.receive_peer_configuration().await?;
        validate_peer_configuration(peer_configuration_b)?;
        validate_old_peer_configuration(old_peer_configuration_b)?;

        receiver_a.expect_no_peer_configuration().await;
        receiver_b.expect_no_peer_configuration().await;
    }
    Ok(())
}

#[test_log::test(
    tokio::test(flavor = "multi_thread")
)]
async fn carl_should_send_cluster_related_peer_configuration_if_a_peer_comes_online_later() -> anyhow::Result<()> {
    let fixture = Fixture::new();

    let carl_port = util::spawn_carl()?;

    let carl_client = TestCarlClient::connect(carl_port).await?;

    let peer_a = store_peer_descriptor(&carl_client).await?;

    let mut receiver_a = util::spawn_edgar_with_peer_configuration_receiver(peer_a.id, carl_port).await?;
    carl_client.await_peer_up(peer_a.id).await?;
    {
        let (peer_configuration_a, old_peer_configuration_a) = receiver_a.receive_peer_configuration().await?;
        assert_eq!(peer_configuration_a, fixture.empty_peer_configuration);
        assert_eq!(old_peer_configuration_a, fixture.empty_old_peer_configuration);
        receiver_a.expect_no_peer_configuration().await;
    }

    let peer_b = store_peer_descriptor(&carl_client).await?;

    let cluster_leader = peer_a.id;
    let cluster_devices = peer_a.topology.devices.iter().chain(peer_b.topology.devices.iter());
    let cluster = store_cluster_configuration(cluster_leader, cluster_devices, &carl_client).await?;

    store_cluster_deployment(cluster.id, &carl_client).await?;
    receiver_a.expect_no_peer_configuration().await;


    let mut receiver_b = util::spawn_edgar_with_peer_configuration_receiver(peer_b.id, carl_port).await?;
    carl_client.await_peer_up(peer_b.id).await?;
    {
        let (peer_configuration_b, old_peer_configuration_b) = receiver_b.receive_peer_configuration().await?;
        assert_eq!(peer_configuration_b, fixture.empty_peer_configuration);
        assert_eq!(old_peer_configuration_b, fixture.empty_old_peer_configuration);
    }

    {
        let validate_peer_configuration = |peer_configuration: PeerConfiguration| {
            assert_that!(peer_configuration, matches_pattern!(PeerConfiguration {
                executors: empty(),
                ethernet_bridges: contains(
                    matches_pattern!(Parameter {
                        id: anything(),
                        dependencies: empty(),
                        target: eq(&ParameterTarget::Present),
                        value: eq(&EthernetBridge {
                            name: NetworkInterfaceName::try_from("br-opendut")?,
                        }),
                    })
                ),
            }));
            Ok::<_, anyhow::Error>(())
        };
        let validate_old_peer_configuration = |old_peer_configuration: OldPeerConfiguration| {
            assert_that!(old_peer_configuration, matches_pattern!(OldPeerConfiguration {
                cluster_assignment: some(matches_pattern!(ClusterAssignment {
                    id: anything(),
                    leader: eq(&cluster_leader),
                    assignments: unordered_elements_are!(
                        matches_pattern!(PeerClusterAssignment {
                            peer_id: eq(&peer_a.id),
                            vpn_address: eq(&IpAddr::from_str("127.0.0.1")?),
                            can_server_port: any!(eq(&Port(10000)), eq(&Port(10001))),
                            device_interfaces: eq(&peer_a.network.interfaces),
                        }),
                        matches_pattern!(PeerClusterAssignment {
                            peer_id: eq(&peer_b.id),
                            vpn_address: eq(&IpAddr::from_str("127.0.0.1")?),
                            can_server_port: any!(eq(&Port(10000)), eq(&Port(10001))),
                            device_interfaces: eq(&peer_b.network.interfaces),
                        }),
                    ),
                }))
            }));
            Ok::<_, anyhow::Error>(())
        };

        let (peer_configuration_a, old_peer_configuration_a) = receiver_a.receive_peer_configuration().await?;
        validate_peer_configuration(peer_configuration_a)?;
        validate_old_peer_configuration(old_peer_configuration_a)?;

        let (peer_configuration_b, old_peer_configuration_b) = receiver_b.receive_peer_configuration().await?;
        validate_peer_configuration(peer_configuration_b)?;
        validate_old_peer_configuration(old_peer_configuration_b)?;

        receiver_a.expect_no_peer_configuration().await;
        receiver_b.expect_no_peer_configuration().await;
    }
    Ok(())
}

struct Fixture {
    empty_peer_configuration: PeerConfiguration,
    empty_old_peer_configuration: OldPeerConfiguration,
}
impl Fixture {
    fn new() -> Self {
        let empty_peer_configuration = PeerConfiguration {
            executors: vec![],
            ethernet_bridges: vec![],
        };
        let empty_old_peer_configuration = OldPeerConfiguration { cluster_assignment: None };

        Fixture { empty_peer_configuration, empty_old_peer_configuration }
    }
}

async fn store_peer_descriptor(carl_client: &TestCarlClient) -> anyhow::Result<PeerDescriptor> {
    let peer_id = PeerId::random();
    let device_id = DeviceId::random();
    let network_interface_id = NetworkInterfaceId::random();

    let peer_descriptor = PeerDescriptor {
        id: peer_id,
        name: PeerName::try_from(format!("peer-{peer_id}"))?,
        location: None,
        network: PeerNetworkDescriptor {
            interfaces: vec![
                NetworkInterfaceDescriptor {
                    id: network_interface_id,
                    name: NetworkInterfaceName::try_from(format!("eth-{short_id}", short_id=network_interface_id.to_string().split("-").next().unwrap()))?,
                    configuration: NetworkInterfaceConfiguration::Ethernet,
                },
            ],
            bridge_name: None,
        },
        topology: Topology {
            devices: vec![
                DeviceDescriptor {
                    id: device_id,
                    name: DeviceName::try_from(format!("device-{device_id}"))?,
                    description: None,
                    interface: network_interface_id,
                    tags: vec![],
                }
            ],
        },
        executors: ExecutorDescriptors {
            executors: vec![],
        },
    };

    carl_client.inner().await.peers
        .store_peer_descriptor(peer_descriptor.clone()).await?;

    Ok(peer_descriptor)
}

async fn store_cluster_configuration(leader: PeerId, devices: impl Iterator<Item=&DeviceDescriptor>, carl_client: &TestCarlClient) -> anyhow::Result<ClusterConfiguration> {
    let cluster_id = ClusterId::random();

    let devices = HashSet::from_iter(
        devices.map(|device| device.id)
    );

    let cluster_configuration = ClusterConfiguration {
        id: cluster_id,
        name: ClusterName::try_from(format!("cluster-{cluster_id}"))?,
        leader,
        devices,
    };

    carl_client.inner().await.cluster.store_cluster_configuration(cluster_configuration.clone()).await?;

    Ok(cluster_configuration)
}

async fn store_cluster_deployment(cluster_id: ClusterId, carl_client: &TestCarlClient) -> anyhow::Result<()> {
    carl_client.inner().await.cluster
        .store_cluster_deployment(ClusterDeployment { id: cluster_id }).await?;
    Ok(())
}
