use crate::testing;
use crate::testing::carl_client::TestCarlClient;
use crate::testing::util;
use googletest::prelude::*;
use opendut_types::cluster::{ClusterAssignment, ClusterConfiguration, ClusterDeployment, ClusterId, ClusterName, PeerClusterAssignment};
use opendut_types::peer::configuration::{OldPeerConfiguration, Parameter, ParameterTarget, PeerConfiguration};
use opendut_types::peer::configuration::parameter;
use opendut_types::peer::PeerId;
use opendut_types::topology::DeviceDescriptor;
use opendut_types::util::net::NetworkInterfaceName;
use opendut_types::util::Port;
use std::collections::HashSet;
use std::net::IpAddr;
use std::str::FromStr;

#[test_log::test(
    tokio::test(flavor = "multi_thread")
)]
async fn carl_should_send_peer_configurations_in_happy_flow() -> anyhow::Result<()> {
    let fixture = Fixture::create().await?;
    let carl = fixture.carl;

    let peer_a = testing::peer_descriptor::store_peer_descriptor(&carl.client).await?;

    let mut receiver_a = util::spawn_edgar_with_peer_configuration_receiver(peer_a.id, carl.port).await?;
    carl.client.await_peer_up(peer_a.id).await?;
    {
        let (peer_configuration_a, old_peer_configuration_a) = receiver_a.receive_peer_configuration().await?;
        assert_eq!(peer_configuration_a, fixture.empty_peer_configuration);
        assert_eq!(old_peer_configuration_a, fixture.empty_old_peer_configuration);
        receiver_a.expect_no_peer_configuration().await;
    }

    let peer_b = testing::peer_descriptor::store_peer_descriptor(&carl.client).await?;

    let mut receiver_b = util::spawn_edgar_with_peer_configuration_receiver(peer_b.id, carl.port).await?;
    carl.client.await_peer_up(peer_b.id).await?;
    {
        let (peer_configuration_b, old_peer_configuration_b) = receiver_b.receive_peer_configuration().await?;
        assert_eq!(peer_configuration_b, fixture.empty_peer_configuration);
        assert_eq!(old_peer_configuration_b, fixture.empty_old_peer_configuration);
        receiver_b.expect_no_peer_configuration().await;
    }

    let cluster_leader = peer_a.id;
    let cluster_devices = peer_a.topology.devices.iter().chain(peer_b.topology.devices.iter());
    let cluster = store_cluster_configuration(cluster_leader, cluster_devices, &carl.client).await?;

    store_cluster_deployment(cluster.id, &carl.client).await?;

    {
        let validate_peer_configuration = |peer_configuration: PeerConfiguration| {
            assert_that!(peer_configuration, matches_pattern!(PeerConfiguration {
                device_interfaces: eq(&peer_configuration.device_interfaces),
                ethernet_bridges: contains(
                    matches_pattern!(Parameter {
                        id: anything(),
                        dependencies: empty(),
                        target: eq(&ParameterTarget::Present),
                        value: eq(&parameter::EthernetBridge {
                            name: NetworkInterfaceName::try_from("br-opendut")?,
                        }),
                    })
                ),
                executors: empty(),
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
                        }),
                        matches_pattern!(PeerClusterAssignment {
                            peer_id: eq(&peer_b.id),
                            vpn_address: eq(&IpAddr::from_str("127.0.0.1")?),
                            can_server_port: any!(eq(&Port(10000)), eq(&Port(10001))),
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
    let fixture = Fixture::create().await?;
    let carl = fixture.carl;

    let peer_a = testing::peer_descriptor::store_peer_descriptor(&carl.client).await?;

    let mut receiver_a = util::spawn_edgar_with_peer_configuration_receiver(peer_a.id, carl.port).await?;
    carl.client.await_peer_up(peer_a.id).await?;
    {
        let (peer_configuration_a, old_peer_configuration_a) = receiver_a.receive_peer_configuration().await?;
        assert_eq!(peer_configuration_a, fixture.empty_peer_configuration);
        assert_eq!(old_peer_configuration_a, fixture.empty_old_peer_configuration);
        receiver_a.expect_no_peer_configuration().await;
    }
    tracing::info!("First invocation of receive_peer_configuration is done.");

    let peer_b = testing::peer_descriptor::store_peer_descriptor(&carl.client).await?;

    let cluster_leader = peer_a.id;
    let cluster_devices = peer_a.topology.devices.iter().chain(peer_b.topology.devices.iter());
    let cluster = store_cluster_configuration(cluster_leader, cluster_devices, &carl.client).await?;

    store_cluster_deployment(cluster.id, &carl.client).await?;
    receiver_a.expect_no_peer_configuration().await;


    let mut receiver_b = util::spawn_edgar_with_peer_configuration_receiver(peer_b.id, carl.port).await?;
    carl.client.await_peer_up(peer_b.id).await?;
    {
        let (peer_configuration_b, old_peer_configuration_b) = receiver_b.receive_peer_configuration().await?;
        assert_eq!(peer_configuration_b, fixture.empty_peer_configuration);
        assert_eq!(old_peer_configuration_b, fixture.empty_old_peer_configuration);
    }
    tracing::info!("Second invocation of receive_peer_configuration is done.");

    {
        let validate_peer_configuration = |peer_configuration: PeerConfiguration| {
            assert_that!(peer_configuration, matches_pattern!(PeerConfiguration {
                device_interfaces: eq(&peer_configuration.device_interfaces),
                ethernet_bridges: contains(
                    matches_pattern!(Parameter {
                        id: anything(),
                        dependencies: empty(),
                        target: eq(&ParameterTarget::Present),
                        value: eq(&parameter::EthernetBridge {
                            name: NetworkInterfaceName::try_from("br-opendut")?,
                        }),
                    })
                ),
                executors: empty(),
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
                        }),
                        matches_pattern!(PeerClusterAssignment {
                            peer_id: eq(&peer_b.id),
                            vpn_address: eq(&IpAddr::from_str("127.0.0.1")?),
                            can_server_port: any!(eq(&Port(10000)), eq(&Port(10001))),
                        }),
                    ),
                }))
            }));
            Ok::<_, anyhow::Error>(())
        };

        tracing::info!("Final peer configuration... ");
        let (peer_configuration_a, old_peer_configuration_a) = receiver_a.receive_peer_configuration().await?;
        validate_peer_configuration(peer_configuration_a)?;
        validate_old_peer_configuration(old_peer_configuration_a)?;
        tracing::info!("receiver_a");

        let (peer_configuration_b, old_peer_configuration_b) = receiver_b.receive_peer_configuration().await?;
        validate_peer_configuration(peer_configuration_b)?;
        validate_old_peer_configuration(old_peer_configuration_b)?;
        tracing::info!("receiver_b");

        receiver_a.expect_no_peer_configuration().await;
        receiver_b.expect_no_peer_configuration().await;
    }
    Ok(())
}

struct Fixture {
    carl: CarlFixture,
    empty_peer_configuration: PeerConfiguration,
    empty_old_peer_configuration: OldPeerConfiguration,
}
impl Fixture {
    async fn create() -> anyhow::Result<Self> {
        let empty_peer_configuration = PeerConfiguration::default();
        let empty_old_peer_configuration = OldPeerConfiguration::default();

        let carl = {
            let port = util::spawn_carl()?;
            let client = TestCarlClient::connect(port).await?;
            CarlFixture { client, port }
        };

        Ok(Fixture {
            carl,
            empty_peer_configuration,
            empty_old_peer_configuration,
        })
    }
}
struct CarlFixture {
    client: TestCarlClient,
    port: Port,
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
