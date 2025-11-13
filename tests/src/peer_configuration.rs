use crate::testing;
use crate::testing::carl_client::TestCarlClient;
use crate::testing::util;
use googletest::prelude::*;
use opendut_model::cluster::{ClusterDescriptor, ClusterDeployment, ClusterId, ClusterName};
use opendut_model::peer::configuration::{Parameter, ParameterField, ParameterTarget, ParameterValue, PeerConfiguration};
use opendut_model::peer::configuration::parameter;
use opendut_model::peer::{PeerDescriptor, PeerId};
use opendut_model::topology::DeviceDescriptor;
use opendut_model::util::net::NetworkInterfaceName;
use opendut_model::util::Port;
use std::collections::HashSet;
use std::net::{IpAddr, Ipv4Addr};
use std::str::FromStr;
use tracing::info;

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
        let peer_configuration_a = receiver_a.receive_peer_configuration().await?;
        assert_eq!(peer_configuration_a, fixture.empty_peer_configuration);
        receiver_a.expect_no_peer_configuration().await;
    }

    let peer_b = testing::peer_descriptor::store_peer_descriptor(&carl.client).await?;

    let mut receiver_b = util::spawn_edgar_with_peer_configuration_receiver(peer_b.id, carl.port).await?;
    carl.client.await_peer_up(peer_b.id).await?;
    {
        let peer_configuration_b = receiver_b.receive_peer_configuration().await?;
        assert_eq!(peer_configuration_b, fixture.empty_peer_configuration);
        receiver_b.expect_no_peer_configuration().await;
    }

    let cluster_leader = peer_a.id;
    let cluster_devices = peer_a.topology.devices.iter().chain(peer_b.topology.devices.iter());
    let cluster = store_cluster_descriptor(cluster_leader, cluster_devices, &carl.client).await?;

    // ACT
    store_cluster_deployment(cluster.id, &carl.client).await?;

    // THEN
    let result_second_store = store_cluster_deployment(cluster.id, &carl.client).await;

    // ASSERT
    assert!(result_second_store.is_ok(), "Storing the same cluster deployment twice should not be a problem. Peer configuration will be sent twice.");
    let peer_configuration_a_first = receiver_a.receive_peer_configuration().await?;
    let peer_configuration_b_first = receiver_b.receive_peer_configuration().await?;

    let peer_configuration_a_second = receiver_a.receive_peer_configuration().await?;
    let peer_configuration_b_second = receiver_b.receive_peer_configuration().await?;
    validate_peer_configuration(peer_a, Some(peer_b.id), peer_configuration_a_second.clone())?;
    validate_peer_configuration(peer_b, None, peer_configuration_b_second.clone())?;

    {
        // compare peer configuration parameters of subsequent sends
        let a_first = serde_json::to_string(&peer_configuration_a_first)?;
        let a_second = serde_json::to_string(&peer_configuration_a_second)?;
        info!("Peer configuration a_first: {}", a_first);
        info!("Peer configuration a_second: {}", a_second);
        let b_first = serde_json::to_string(&peer_configuration_b_first)?;
        let b_second = serde_json::to_string(&peer_configuration_b_second)?;
        info!("Peer configuration b_first: {}", b_first);
        info!("Peer configuration b_second: {}", b_second);

        assert_eq!(peer_configuration_a_first, peer_configuration_a_second, "Peer A received peer configuration that does not match on repeated send.");
        assert_eq!(peer_configuration_b_first, peer_configuration_b_second, "Peer B received peer configuration that does not match on repeated send.");

    }
    {
        receiver_a.expect_no_peer_configuration().await;
        receiver_b.expect_no_peer_configuration().await;
    }

    Ok(())
}

fn validate_peer_configuration(peer_descriptor: PeerDescriptor, check_remote_peer_id: Option<PeerId>, peer_configuration: PeerConfiguration) -> anyhow::Result<()> {
    let bridge = parameter::EthernetBridge { name: NetworkInterfaceName::try_from("br-opendut").expect("Could not construct interface name.") };
    let ethernet_descriptor = peer_descriptor.network.interfaces.first().cloned().expect("Peer has no network interfaces.");
    let ethernet = parameter::DeviceInterface { descriptor: ethernet_descriptor.clone() };
    let gre_interface = parameter::GreInterfaceConfig {
        local_ip: Ipv4Addr::from_str("127.0.0.1")?,
        remote_ip: Ipv4Addr::from_str("127.0.0.1")?,
    };
    let interface_join_gre = parameter::InterfaceJoinConfig {
        name: gre_interface.interface_name()?,
        bridge: bridge.name.clone(),
    };
    let interface_join_ethernet = parameter::InterfaceJoinConfig {
        name: ethernet_descriptor.name,
        bridge: bridge.name.clone(),
    };
    match check_remote_peer_id {
        Some(remote_peer_id) => {
            // leader does remote peer connection check
            let remote_peer_check = parameter::RemotePeerConnectionCheck {
                remote_peer_id,
                remote_ip: IpAddr::V4(Ipv4Addr::from_str("127.0.0.1")?),
            };
            assert_that!(peer_configuration.remote_peer_connection_checks, matches_pattern!(ParameterField {
                values: has_entry(
                    remote_peer_check.parameter_identifier(),
                    matches_pattern!(Parameter {
                        id: eq(&remote_peer_check.parameter_identifier()),
                        dependencies: is_empty(),
                        target: eq(&ParameterTarget::Present),
                        value: eq(&remote_peer_check),
                        ..
                    })
                ),
            }) );
        }
        None => {
            // other peers do not perform remote peer connection checks
            assert_that!(peer_configuration.remote_peer_connection_checks, matches_pattern!(ParameterField {
                values: is_empty(),
            }));
        }
    }

    assert_that!(peer_configuration, matches_pattern!(PeerConfiguration {
                device_interfaces: matches_pattern!(ParameterField {
                    values: has_entry(
                        ethernet.parameter_identifier(),
                        matches_pattern!(Parameter {
                            id: anything(),
                            dependencies: is_empty(),
                            target: eq(&ParameterTarget::Present),
                            value: eq(&ethernet),
                            ..
                        })
                    ),
                }),
                ethernet_bridges: matches_pattern!(ParameterField {
                    values: has_entry(
                        bridge.parameter_identifier(),
                        matches_pattern!(Parameter {
                            id: anything(),
                            dependencies: is_empty(),
                            target: eq(&ParameterTarget::Present),
                            value: eq(&bridge),
                            ..
                        })
                    ),
                }),
                executors: matches_pattern!(ParameterField {
                    values: is_empty(),
                }),
                gre_interfaces: matches_pattern!(ParameterField {
                    values: has_entry(
                        gre_interface.parameter_identifier(),
                        matches_pattern!(Parameter {
                            id: anything(),
                            dependencies: unordered_elements_are!(eq(&ethernet.parameter_identifier()), eq(&bridge.parameter_identifier())),
                            target: eq(&ParameterTarget::Present),
                            value: eq(&gre_interface),
                            ..
                        })
                    ),
                }),
                joined_interfaces: matches_pattern!(ParameterField {
                    values: unordered_elements_are!(
                        (eq(&interface_join_gre.parameter_identifier()), matches_pattern!(Parameter {
                            id: &interface_join_gre.parameter_identifier(),
                            dependencies: unordered_elements_are!(eq(&gre_interface.parameter_identifier()), eq(&bridge.parameter_identifier())),
                            target: eq(&ParameterTarget::Present),
                            value: eq(&interface_join_gre),
                            ..
                        })),
                        (eq(&interface_join_ethernet.parameter_identifier()), matches_pattern!(Parameter {
                            id: &interface_join_ethernet.parameter_identifier(),
                            dependencies: unordered_elements_are!(eq(&gre_interface.parameter_identifier()), eq(&bridge.parameter_identifier())),
                            target: eq(&ParameterTarget::Present),
                            value: eq(&interface_join_ethernet),
                            ..
                        })),
                    ),
                }),
                // CAN connection related parameters should be empty in this test
                can_connections: matches_pattern!(ParameterField {
                    values: is_empty(),
                }),
                can_bridges: matches_pattern!(ParameterField {
                    values: is_empty(),
                }),
                can_local_routes: matches_pattern!(ParameterField {
                    values: is_empty(),
                }),
                ..
            }));
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
    carl.client.await_peer_up(peer_a.id).await?; // Peer A comes online
    {
        let peer_configuration_a = receiver_a.receive_peer_configuration().await?;
        assert_eq!(peer_configuration_a, fixture.empty_peer_configuration);
        receiver_a.expect_no_peer_configuration().await;
    }

    let peer_b = testing::peer_descriptor::store_peer_descriptor(&carl.client).await?;

    // ACT
    {
        let cluster_leader = peer_a.id;
        let cluster_devices = peer_a.topology.devices.iter().chain(peer_b.topology.devices.iter());
        let cluster = store_cluster_descriptor(cluster_leader, cluster_devices, &carl.client).await?;
        store_cluster_deployment(cluster.id, &carl.client).await?;
    }
    // ASSERT
    receiver_a.expect_no_peer_configuration().await;  // No configuration sent yet

    // ACT
    let mut receiver_b = util::spawn_edgar_with_peer_configuration_receiver(peer_b.id, carl.port).await?;
    {
        carl.client.await_peer_up(peer_b.id).await?; // Peer B comes online later
        let peer_configuration_b = receiver_b.receive_peer_configuration().await?;
        assert_eq!(peer_configuration_b, fixture.empty_peer_configuration);
    }

    {
        // ASSERT
        let peer_configuration_a = receiver_a.receive_peer_configuration().await?;
        validate_peer_configuration(peer_a, Some(peer_b.id), peer_configuration_a)?;

        let peer_configuration_b = receiver_b.receive_peer_configuration().await?;
        validate_peer_configuration(peer_b, None, peer_configuration_b)?;

        receiver_a.expect_no_peer_configuration().await;
        receiver_b.expect_no_peer_configuration().await;
    }
    Ok(())
}

struct Fixture {
    carl: CarlFixture,
    empty_peer_configuration: PeerConfiguration,
}
impl Fixture {
    async fn create() -> anyhow::Result<Self> {
        let empty_peer_configuration = PeerConfiguration::default();

        let carl = {
            let port = util::spawn_carl()?;
            let client = TestCarlClient::connect(port).await?;
            CarlFixture { client, port }
        };

        Ok(Fixture {
            carl,
            empty_peer_configuration,
        })
    }
}
struct CarlFixture {
    client: TestCarlClient,
    port: Port,
}

async fn store_cluster_descriptor(leader: PeerId, devices: impl Iterator<Item=&DeviceDescriptor>, carl_client: &TestCarlClient) -> anyhow::Result<ClusterDescriptor> {
    let cluster_id = ClusterId::random();

    let devices = HashSet::from_iter(
        devices.map(|device| device.id)
    );

    let cluster_descriptor = ClusterDescriptor {
        id: cluster_id,
        name: ClusterName::try_from(format!("cluster-{cluster_id}"))?,
        leader,
        devices,
    };

    carl_client.inner().await.cluster.store_cluster_descriptor(cluster_descriptor.clone()).await?;

    Ok(cluster_descriptor)
}

async fn store_cluster_deployment(cluster_id: ClusterId, carl_client: &TestCarlClient) -> anyhow::Result<()> {
    carl_client.inner().await.cluster
        .store_cluster_deployment(ClusterDeployment { id: cluster_id }).await?;
    Ok(())
}
