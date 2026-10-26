use cli_table::{print_stdout, Table, WithTitle};
use serde::Serialize;

use opendut_carl_api::carl::CarlClient;
use opendut_model::peer::{PeerDescriptor, PeerId, PeerName};
use opendut_model::peer::state::PeerState;
use crate::commands::peer::PeerStatus;
use crate::OutputFormat;

/// List all peers
#[derive(clap::Parser)]
pub struct ListPeersCli;

#[derive(Serialize, Debug)]
struct SerializablePeer {
    name: PeerName,
    id: PeerId,
    status: PeerStatus,
}

#[derive(Table)]
struct PeerTable {
    #[table(title = "Name")]
    name: PeerName,
    #[table(title = "PeerID")]
    id: PeerId,
    #[table(title = "Status")]
    status: PeerStatus,
}

impl From<SerializablePeer> for PeerTable {
    fn from(peer: SerializablePeer) -> Self {
        let SerializablePeer { name, id, status } = peer;

        PeerTable {
            name,
            id,
            status,
        }
    }
}

impl ListPeersCli {
    pub async fn execute(self, carl: &mut CarlClient, output: OutputFormat) -> crate::Result<()> {
        let all_peer_descriptors = carl.peers.list_peer_descriptors().await
            .map_err(|error| format!("Could not list peers.\n  {error}"))?;

        let all_peer_states = carl.peers.list_peer_states().await
            .map_err(|_| "Failed to list peer states!")?;
        
        let serializable_peers = all_peer_descriptors.into_iter()
            .map(|peer| {
                let peer_state = all_peer_states.get(&peer.id).cloned().unwrap_or_default();
                add_peer_status(&peer, peer_state)
            })
            .collect::<Vec<_>>();

        match output {
            OutputFormat::Table => {
                let peer_table = serializable_peers.into_iter()
                    .map(PeerTable::from)
                    .collect::<Vec<_>>();

                print_stdout(peer_table.with_title())
                    .expect("List of clusters should be printable as table.");
            }
            OutputFormat::Json => {
                let json = serde_json::to_string(&serializable_peers).unwrap();
                println!("{json}");
            }
            OutputFormat::PrettyJson => {
                let json = serde_json::to_string_pretty(&serializable_peers).unwrap();
                println!("{json}");
            }
        }
        Ok(())
    }
}

fn add_peer_status(
    peer: &PeerDescriptor,
    peer_state: PeerState
) -> SerializablePeer {
    let status = peer_state.into();

    SerializablePeer {
        name: Clone::clone(&peer.name),
        id: peer.id,
        status,
    }
}

#[cfg(test)]
mod test {
    use googletest::prelude::*;

    use opendut_model::cluster::ClusterId;
    use opendut_model::peer::{PeerDescriptor, PeerId, PeerLocation, PeerName, PeerNetworkDescriptor};
    use opendut_model::peer::state::{PeerConnectionState, PeerMemberState};
    use opendut_model::util::net::{NetworkInterfaceConfiguration, NetworkInterfaceDescriptor, NetworkInterfaceId, NetworkInterfaceName};
    use crate::commands::peer::PeerStatus;
    use super::*;

    #[test]
    fn should_add_peer_status_from_state() {
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
        };
        let cluster_id = ClusterId::random();
        let blocked = PeerMemberState::Blocked { by_cluster: cluster_id };
        let online = PeerConnectionState::Online { remote_host: "127.0.0.1".parse().unwrap() };

        for (peer_state, expected_status) in [
            (PeerState::default(), PeerStatus::Offline),
            (PeerState { connection: PeerConnectionState::Offline, member: blocked.clone() }, PeerStatus::Offline),
            (PeerState { connection: online.clone(), member: PeerMemberState::Available }, PeerStatus::Available),
            (PeerState { connection: online, member: blocked }, PeerStatus::Blocked(cluster_id)),
        ] {
            let serializable_peer = add_peer_status(&peer, peer_state);
            assert_that!(
                serializable_peer,
                matches_pattern!(SerializablePeer {
                    name: eq(&peer.name),
                    id: eq(&peer.id),
                    status: eq(&expected_status),
                    ..
                }),
            );
        }
    }
}
