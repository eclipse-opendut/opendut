pub mod list {
    use std::fmt::{Display, Formatter};

    use cli_table::{print_stdout, Table, WithTitle};
    use serde::Serialize;

    use opendut_carl_api::carl::CarlClient;
    use opendut_types::peer::{PeerDescriptor, PeerId, PeerLocation, PeerName};

    use crate::ListOutputFormat;

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

    pub async fn execute(carl: &mut CarlClient, output: ListOutputFormat) -> crate::Result<()> {
        let connected_peers = carl
            .broker
            .list_peers()
            .await
            .map_err(|error| format!("Could not list connected peers. {}", error))?;
        let all_peers = carl
            .peers
            .list_peer_descriptors()
            .await
            .map_err(|error| format!("Could not list peers.\n  {}", error))?;
        let peers_table = filter_connected_peers(&all_peers, &connected_peers);

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

    fn filter_connected_peers(
        all_peers: &[PeerDescriptor],
        connected_peers: &[PeerId],
    ) -> Vec<PeerTable> {
        all_peers
            .iter()
            .map(|peer| {
                let status = if connected_peers.contains(&peer.id) {
                    PeerStatus::Connected
                } else {
                    PeerStatus::Disconnected
                };
                PeerTable {
                    name: Clone::clone(&peer.name),
                    id: peer.id,
                    location: Clone::clone(&peer.location.clone().unwrap_or_default()),
                    status,
                }
            })
            .collect::<Vec<PeerTable>>()
    }

    #[cfg(test)]
    mod test {
        use googletest::prelude::*;

        use opendut_types::peer::{PeerDescriptor, PeerId, PeerLocation, PeerName};

        use super::*;

        #[test]
        fn test() {
            let all_peers = vec![PeerDescriptor {
                id: PeerId::random(),
                name: PeerName::try_from("MyPeer").unwrap(),
                location: Some(PeerLocation::try_from("SiFi").unwrap()),
                topology: Default::default(),
            }];
            let connected_peers = vec![all_peers[0].id];
            assert_that!(
                filter_connected_peers(&all_peers, &connected_peers),
                unordered_elements_are!(matches_pattern!(PeerTable {
                    name: eq(Clone::clone(&all_peers[0].name)),
                    id: eq(all_peers[0].id),
                    status: eq(PeerStatus::Connected),
                }))
            );
        }
    }
}

pub mod describe {
    use indoc::indoc;
    use uuid::Uuid;

    use opendut_carl_api::carl::CarlClient;
    use opendut_types::peer::{PeerDescriptor, PeerId};

    use crate::DescribeOutputFormat;

    pub async fn execute(
        carl: &mut CarlClient,
        peer_id: Uuid,
        output: DescribeOutputFormat,
    ) -> crate::Result<()> {
        let peer_id = PeerId::from(peer_id);

        let peer_descriptor =
            carl.peers.get_peer_descriptor(peer_id).await.map_err(|_| {
                format!("Failed to retrieve peer descriptor for peer <{}>", peer_id)
            })?;

        render_peer_descriptor(peer_descriptor, output);
        Ok(())
    }

    pub fn render_peer_descriptor(peer_descriptor: PeerDescriptor, output: DescribeOutputFormat) {
        let peer_devices = peer_descriptor
            .topology
            .devices
            .iter()
            .map(|device| device.name.value())
            .collect::<Vec<_>>()
            .join(", ");
        let text = match output {
            DescribeOutputFormat::Text => {
                format!(
                    indoc!(
                        "
                    Peer: {}
                      Id: {}
                      Devices: [{}]\
                "
                    ),
                    peer_descriptor.name, peer_descriptor.id, peer_devices
                )
            }
            DescribeOutputFormat::Json => serde_json::to_string(&peer_descriptor).unwrap(),
            DescribeOutputFormat::PrettyJson => {
                serde_json::to_string_pretty(&peer_descriptor).unwrap()
            }
        };
        println!("{text}");
    }
}

pub mod generate_peer_setup {
    use opendut_carl_api::carl::CarlClient;
    use opendut_types::peer::PeerId;
    use uuid::Uuid;

    //TODO: what happens if peer with the ID is already set up?
    pub async fn execute(carl: &mut CarlClient, id: Uuid) -> crate::Result<()> {
        let peer_id = PeerId::from(id);
        let created_setup = carl
            .peers
            .create_peer_setup(peer_id)
            .await
            .map_err(|error| format!("Could not create peer setup.\n  {}", error))?;

        match created_setup.encode() {
            Ok(setup_key) => {
                println!("{}", setup_key);
            }
            Err(_) => {
                println!("Could not configure setup key...")
            }
        }
        Ok(())
    }
}

pub mod decode_peer_setup {
    use crate::DecodePeerSetupOutputFormat;
    use opendut_types::peer::PeerSetup;

    pub async fn execute(
        setup: PeerSetup,
        output: DecodePeerSetupOutputFormat,
    ) -> crate::Result<()> {
        let text = match output {
            DecodePeerSetupOutputFormat::Text => {
                format!("{:#?}", setup)
            }
            DecodePeerSetupOutputFormat::Json => serde_json::to_string(&setup).unwrap(),
            DecodePeerSetupOutputFormat::PrettyJson => {
                serde_json::to_string_pretty(&setup).unwrap()
            }
        };
        println!("{text}");
        Ok(())
    }
}

pub mod create {
    use console::Style;
    use uuid::Uuid;

    use crate::CreateOutputFormat;
    use opendut_carl_api::carl::CarlClient;
    use opendut_types::peer::{PeerDescriptor, PeerId, PeerLocation, PeerName};

    pub async fn execute(
        carl: &mut CarlClient,
        name: String,
        id: Option<Uuid>,
        location: Option<String>,
        output: CreateOutputFormat,
    ) -> crate::Result<()> {
        let id = PeerId::from(id.unwrap_or_else(Uuid::new_v4));

        let name = PeerName::try_from(name)
            .map_err(|error| format!("Could not create peer.\n  {}", error))?;

        let location = location
            .map(PeerLocation::try_from)
            .transpose()
            .map_err(|error| format!("Could not create peer.\n  {}", error))?;

        let descriptor: PeerDescriptor = PeerDescriptor {
            id,
            name: Clone::clone(&name),
            location,
            topology: Default::default(),
        };
        carl.peers
            .store_peer_descriptor(descriptor.clone())
            .await
            .map_err(|error| format!("Failed to create new peer.\n  {error}"))?;
        let bold = Style::new().bold();
        match output {
            CreateOutputFormat::Text => {
                println!(
                    "Created the peer '{}' with the ID: <{}>",
                    name,
                    bold.apply_to(id)
                );
            }
            CreateOutputFormat::Json => {
                let json = serde_json::to_string(&descriptor).unwrap();
                println!("{}", json);
            }
            CreateOutputFormat::PrettyJson => {
                let json = serde_json::to_string_pretty(&descriptor).unwrap();
                println!("{}", json);
            }
        }
        Ok(())
    }
}

pub mod delete {
    use uuid::Uuid;

    use opendut_carl_api::carl::CarlClient;
    use opendut_types::peer::PeerId;

    pub async fn execute(carl: &mut CarlClient, id: Uuid) -> crate::Result<()> {
        let id = PeerId::from(id);
        carl.peers
            .delete_peer_descriptor(id)
            .await
            .map_err(|error| format!("Failed to delete peer with the id '{}'.\n  {}", id, error))?;
        println!("Deleted peer with the PeerID: {}", id);

        Ok(())
    }
}
