use opendut_model::peer::{PeerDescriptor, PeerId, PeerName, PeerNetworkDescriptor};
use opendut_model::peer::executor::ExecutorDescriptors;
use opendut_model::topology::{DeviceDescriptor, DeviceId, DeviceName, Topology};
use opendut_model::util::net::{NetworkInterfaceConfiguration, NetworkInterfaceDescriptor, NetworkInterfaceId, NetworkInterfaceName};
use crate::testing::carl_client::TestCarlClient;

pub async fn store_peer_descriptor(carl_client: &TestCarlClient) -> anyhow::Result<PeerDescriptor> {
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
