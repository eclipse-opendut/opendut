use leptos::prelude::*;
use opendut_lea_components::{SelectionTable, SelectionTableRow};
use opendut_model::peer::{PeerDescriptor, PeerId};

use crate::clusters::configurator::components::get_all_selected_devices;
use crate::clusters::configurator::types::UserClusterDescriptor;
use crate::components::Ior;

pub type LeaderSelectionError = String;
pub type LeaderSelection = Ior<LeaderSelectionError, PeerId>;

#[component]
pub fn LeaderSelector(
    user_cluster_descriptor: RwSignal<UserClusterDescriptor>,
    peers: ReadSignal<Vec<PeerDescriptor>>,
    is_disabled: Signal<bool>,
) -> impl IntoView {

    let getter_selected_devices = create_read_slice(user_cluster_descriptor, |config| {
        Clone::clone(&config.devices)
    });

    let (getter, setter) = create_slice(
        user_cluster_descriptor,
        |config| Clone::clone(&config.leader),
        |config, input| {
            config.leader = input;
        },
    );

    let selected_devices = Signal::derive(move || get_all_selected_devices(getter_selected_devices));

    Effect::new(move || {
        let selected_devices = selected_devices.get();
        let selection = getter.get();
        let error = if selected_devices.len() < 2 {
            Some("Please select at least two devices first.")
        } else {
            let has_eligible_leader = match &selection {
                LeaderSelection::Right(leader) => peers.with(|peers| {
                    peers.iter().any(|peer| {
                        peer.id == *leader && peer.topology.devices.iter()
                            .any(|device| selected_devices.contains(&device.id))
                    })
                }),
                LeaderSelection::Left(_) | LeaderSelection::Both(_, _) => false,
            };

            (!has_eligible_leader).then_some("Select a leader.")
        };

        if let Some(error) = error {
            let validated_selection = LeaderSelection::Left(String::from(error));
            if selection != validated_selection {
                setter.set(validated_selection);
            }
        }
    });

    let peers = Signal::derive(move || {
        let selected_devices = selected_devices.get();
        let mut peers = peers.get();

        peers.sort_by(|a, b| {
            a.name.value().to_lowercase()
                .cmp(&b.name.value().to_lowercase())
        });

        peers.into_iter()
            .filter(|peer_descriptor| {
                peer_descriptor.topology.devices.iter()
                    .any(|device| selected_devices.contains(&device.id))
            }).map(|peer_descriptor| {
                let PeerDescriptor { id, name, location, .. } = peer_descriptor;
                let name = name.value().to_owned();
                let location = location.unwrap_or_default().to_string();

                SelectionTableRow {
                    id,
                    cells: vec![name.into(), id.to_string().into(), location.into()]
                }
            })
            .collect::<Vec<_>>()
    });

    let header = vec![
        String::from("Leader"),
        String::from("Name"),
        String::from("Peer ID"),
        String::from("Location"),
    ];

    view! {
        <SelectionTable
            header
            rows=peers
            getter
            setter
            is_disabled
        />
    }
}
