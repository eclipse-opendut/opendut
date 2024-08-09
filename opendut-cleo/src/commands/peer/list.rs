use std::fmt::{Display, Formatter};

use cli_table::{print_stdout, Table, WithTitle};
use serde::Serialize;

use opendut_carl_api::carl::CarlClient;
use opendut_types::peer::{PeerDescriptor, PeerId, PeerLocation, PeerName};
use opendut_types::peer::state::PeerState;
use crate::ListOutputFormat;

/// List all peers
#[derive(clap::Parser)]
pub struct ListPeersCli;

#[derive(Table, Debug, Serialize)]
struct PeerTable {
    #[table(title = "Name")]
    name: PeerName,
    #[table(title = "PeerID")]
    id: PeerId,
    #[table(title = "Status")]
    status: PeerStatus,
    #[table(title = "Location")]
    location: PeerLocation,
    #[table(title = "NetworkInterfaces")]
    network_interfaces: String,
}

#[derive(Debug, PartialEq, Serialize)]
enum PeerStatus {
    Connected,
    Disconnected,
}

impl Display for PeerStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PeerStatus::Connected => write!(f, "Connected"),
            PeerStatus::Disconnected => write!(f, "Disconnected"),
        }
    }
}

impl ListPeersCli {
    pub async fn execute(self, carl: &mut CarlClient, output: ListOutputFormat) -> crate::Result<()> {
        let all_peers = carl
            .peers
            .list_peer_descriptors()
            .await
            .map_err(|error| format!("Could not list peers.\n  {}", error))?;
        
        let mut peers_table = vec![];
        for peer in all_peers {
            let peer_state = carl.peers.get_peer_state(peer.id).await.map_err(|_| {
                format!("Failed to retrieve state for peer <{}>", peer.id)
            })?;
            peers_table.push(add_peer_status(peer, peer_state));
        };
        match output {
            ListOutputFormat::Table => {
                print_stdout(peers_table.with_title())
                    .expect("List of clusters should be printable as table.");
            }
            ListOutputFormat::Json => {
                let json = serde_json::to_string(&peers_table).unwrap();
                println!("{}", json);
            }
            ListOutputFormat::PrettyJson => {
                let json = serde_json::to_string_pretty(&peers_table).unwrap();
                println!("{}", json);
            }
        }
        Ok(())
    }
}

fn add_peer_status(
    peer: PeerDescriptor,
    peer_state: PeerState
) -> PeerTable {
    let status = match peer_state {
        PeerState::Down => { PeerStatus::Disconnected }
        PeerState::Up { .. } => { PeerStatus::Connected }
    };
    let network_interfaces = Clone::clone(&peer.network.interfaces);
    let interfaces = network_interfaces.into_iter().map(|interface| interface.name.to_string()).collect::<Vec<_>>();
    PeerTable {
        name: Clone::clone(&peer.name),
        id: peer.id,
        location: Clone::clone(&peer.location.clone().unwrap_or_default()),
        network_interfaces: interfaces.join(", "),
        status
    }
}

#[cfg(test)]
mod test {
    use googletest::prelude::*;

    use opendut_types::peer::{PeerDescriptor, PeerId, PeerLocation, PeerName, PeerNetworkDescriptor};
    use opendut_types::peer::executor::ExecutorDescriptors;
    use opendut_types::util::net::{NetworkInterfaceConfiguration, NetworkInterfaceDescriptor, NetworkInterfaceId, NetworkInterfaceName};

    use super::*;

    #[test]
    fn test() {
        let peer = PeerDescriptor {
            id: PeerId::random(),
            name: PeerName::try_from("MyPeer").unwrap(),
            location: Some(PeerLocation::try_from("SiFi").unwrap()),
            network: PeerNetworkDescriptor{
                interfaces: vec!(NetworkInterfaceDescriptor {
                    id: NetworkInterfaceId::random(),
                    name: NetworkInterfaceName::try_from("eth0").unwrap(),
                    configuration: NetworkInterfaceConfiguration::Ethernet,
                }),
                bridge_name: Some(NetworkInterfaceName::try_from("br-opendut-1").unwrap())
            },
            topology: Default::default(),
            executors: ExecutorDescriptors {
                executors: vec![]
            }
        };
        assert_that!(
            add_peer_status(peer.clone(), PeerState::Down),
            matches_pattern!(PeerTable {
                name: eq(Clone::clone(&peer.name)),
                id: eq(peer.id),
                status: eq(PeerStatus::Disconnected),
            })
        );
    }
}
