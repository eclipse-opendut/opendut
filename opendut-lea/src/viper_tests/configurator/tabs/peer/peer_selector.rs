use leptos::prelude::*;
use opendut_lea_components::{SelectionTable, SelectionTableRow};
use opendut_model::peer::PeerDescriptor;
use crate::app::use_app_globals;
use crate::viper_tests::configurator::types::{PeerSelection, UserViperTestRunDescriptor};

#[component]
pub fn PeerSelector(viper_test_run_descriptor: RwSignal<UserViperTestRunDescriptor>) -> impl IntoView {

    let globals = use_app_globals();

    let list_of_peers = {
        let carl = globals.client.clone();

        LocalResource::new(move || {
            let mut carl = carl.clone();
            async move {
                carl.peers.list_peer_descriptors().await
                    .expect("Failed to request the list of peers")
            }
        })
    };

    let (getter, setter) = create_slice(viper_test_run_descriptor,
        |config| {
            Clone::clone(&config.peer)
        },
        |config, input| {
            config.peer = input;
        }
    );

    let peers = Signal::derive(move || {
        if let Some(mut peers) = list_of_peers.get() {
            peers
                .sort_by(|peer_a, peer_b| {
                    peer_a.name.value().to_lowercase()
                        .cmp(&peer_b.name.value().to_lowercase())
                });

            let rows = peers.iter().map(|peer_descriptor| {
                let PeerDescriptor { id, name, location: _location, network: _network, topology: _topology, executors: _executors } = peer_descriptor;
                let id = id.to_owned();
                let name = name.value().to_owned();

                SelectionTableRow {
                    id: Clone::clone(&id),
                    cells: vec![name, id.to_string()]
                }
            }).collect::<Vec<_>>();

            if peers.is_empty() {
                setter.set(PeerSelection::Left(String::from("No peers available.")));
            } else if matches!(getter.get(), PeerSelection::Left(_)) {
                setter.set(PeerSelection::Left(String::from("Select a peer.")));
            }

            rows
        } else {
            Vec::new()
        }
    });

    let header = vec![
        String::new(),
        String::from("Name"),
        String::from("Peer ID")
    ];

    view! {
        <SelectionTable
            header
            rows=peers
            getter
            setter
        />
    }
}
