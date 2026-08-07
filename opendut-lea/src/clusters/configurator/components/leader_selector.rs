use std::collections::HashSet;

use leptos::prelude::*;
use opendut_lea_components::{SelectionTable, SelectionTableRow};
use opendut_model::peer::{PeerDescriptor, PeerId};
use opendut_model::topology::DeviceId;

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

    let selected_devices = move || get_all_selected_devices(getter_selected_devices);

    let peers = Signal::derive(move || {
        let selected_devices = selected_devices();
        let mut peers = peers.get();

        peers.sort_by(|a, b| {
            a.name.value().to_lowercase()
                .cmp(&b.name.value().to_lowercase())
        });

        peers.clone().into_iter()
            .filter(|peer_descriptor| {
                let mut peer_devices: HashSet<DeviceId> = HashSet::new();

                for device in &peer_descriptor.topology.devices {
                    peer_devices.insert(device.id);
                }

                if selected_devices.len() < 2 {
                    setter.set(LeaderSelection::Left(String::from("Please select at least two devices first.")));
                }
                else {
                    let leader_not_selected = match getter.get() {
                        LeaderSelection::Left(_) | LeaderSelection::Both(_, _) => true,
                        LeaderSelection::Right(leader) => {
                            // Deselecting a previously selected peer leader in case all devices belonging to the peer were also deselected
                            peer_devices.is_disjoint(&selected_devices) && peer_descriptor.id == leader
                        }
                    };

                    if leader_not_selected {
                        setter.set(LeaderSelection::Left(String::from("Select a leader.")));
                    }
                }

                !peer_devices.is_disjoint(&selected_devices)
            }).map(|peer_descriptor| {
                let PeerDescriptor { id, name, location, .. } = peer_descriptor;
                let name = name.value().to_owned();
                let location = location.unwrap_or_default().to_string();

                SelectionTableRow {
                    id: Clone::clone(&id),
                    cells: vec![name, id.to_string(), location]
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
