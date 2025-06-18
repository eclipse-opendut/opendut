use opendut_types::peer::{PeerDescriptor, PeerId, PeerLocation, PeerName, PeerNetworkDescriptor};
use opendut_types::peer::executor::ExecutorDescriptors;
use opendut_types::topology::Topology;
use opendut_types::util::net::NetworkInterfaceName;

pub fn create_peer_descriptor(peer_id: PeerId) -> PeerDescriptor {
    PeerDescriptor {
        id: peer_id,
        name: PeerName::try_from("PeerA").unwrap(),
        location: PeerLocation::try_from("Ulm").ok(),
        network: PeerNetworkDescriptor {
            interfaces: vec![],
            bridge_name: Some(NetworkInterfaceName::try_from("br-opendut-1").unwrap()),
        },
        topology: Topology {
            devices: vec![],
        },
        executors: ExecutorDescriptors {
            executors: vec![],
        }
    }
}
