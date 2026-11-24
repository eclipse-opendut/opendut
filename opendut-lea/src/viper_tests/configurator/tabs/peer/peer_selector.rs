use leptos::prelude::*;
use opendut_lea_components::{ErrorText, SelectionTable, SelectionTableCell, SelectionTableRow};
use opendut_model::cluster::ClusterDescriptor;
use opendut_model::peer::PeerDescriptor;
use crate::util;
use crate::viper_tests::configurator::types::{PeerSelection, UserViperTestRunDescriptor};

#[component]
pub fn PeerSelector(
    user_test_run_descriptor: RwSignal<UserViperTestRunDescriptor>,
    peers: ReadSignal<Vec<PeerDescriptor>>,
    clusters: ReadSignal<Result<Vec<ClusterDescriptor>, ErrorText>>,
) -> impl IntoView {

    let (getter, setter) = create_slice(user_test_run_descriptor,
        |config| {
            Clone::clone(&config.peer)
        },
        |config, input| {
            config.peer = input;
        }
    );

    Effect::new(move || {
        let selection = getter.get();
        let error = if peers.with(Vec::is_empty) {
            Some("No peers available.")
        } else if matches!(selection, PeerSelection::Left(_)) {
            Some("Select a peer.")
        } else {
            None
        };

        if let Some(error) = error {
            let validated_selection = PeerSelection::Left(String::from(error));
            if selection != validated_selection {
                setter.set(validated_selection);
            }
        }
    });

    let rows = Signal::derive(move || {
        peers.get().iter().map(|peer_descriptor| {
            let PeerDescriptor { id, name, location: _location, network: _network, topology } = peer_descriptor;
            let id = id.to_owned();
            let name = name.value().to_owned();

            let devices_in_peer = topology
                .devices
                .iter()
                .map(|device| device.id)
                .collect::<Vec<_>>();

            let clusters = match clusters.get() {
                Ok(clusters) => {
                    let cluster_list = util::list_configured_clusters_for_peer(devices_in_peer, clusters)
                        .iter()
                        .map(|cluster| cluster.name.value().to_owned())
                        .collect::<Vec<_>>()
                        .join(", ");
                    SelectionTableCell::Text(cluster_list)
                }
                Err(error) => {
                    let tip = error.headline;
                    SelectionTableCell::Error(tip)
                }
            };

            SelectionTableRow {
                id,
                cells: vec![name.into(), id.to_string().into(), clusters]
            }
        }).collect::<Vec<_>>()
    });

    let header = vec![
        String::new(),
        String::from("Name"),
        String::from("Peer ID"),
        String::from("Configured in Clusters"),
    ];

    view! {
        <SelectionTable
            header
            rows
            getter
            setter
        />
    }
}
