use std::collections::HashSet;

use leptos::prelude::*;

use opendut_types::peer::{PeerDescriptor, PeerId};
use opendut_types::topology::DeviceId;

use crate::clusters::configurator::components::get_all_selected_devices;
use crate::clusters::configurator::types::UserClusterDescriptor;
use crate::components::{Ior, NON_BREAKING_SPACE};

pub type LeaderSelectionError = String;
pub type LeaderSelection = Ior<LeaderSelectionError, PeerId>;

#[component]
pub fn LeaderSelector(
    cluster_descriptor: RwSignal<UserClusterDescriptor>,
    peers: ReadSignal<Vec<PeerDescriptor>>,
) -> impl IntoView {

    let getter_selected_devices = create_read_slice(cluster_descriptor, |config| {
        Clone::clone(&config.devices)
    });

    let (getter_leader, setter_leader) = create_slice(
        cluster_descriptor,
        |config| Clone::clone(&config.leader),
        |config, input| {
            config.leader = input;
        },
    );

    let selected_devices = move || get_all_selected_devices(getter_selected_devices);

    let help_text = move || {
        getter_leader.with(|selection| match selection {
            LeaderSelection::Right(_) => String::from(NON_BREAKING_SPACE),
            LeaderSelection::Left(error) => error.to_owned(),
            LeaderSelection::Both(error, _) => error.to_owned(),
        })
    };

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
                    setter_leader.set(LeaderSelection::Left(String::from("Please select at least two devices first.")));
                }
                else {
                    let leader_not_selected = match getter_leader.get() {
                        LeaderSelection::Left(_) | LeaderSelection::Both(_, _) => true,
                        LeaderSelection::Right(leader) => {
                            // Deselecting a previously selected peer leader in case all devices belonging to the peer were also deselected
                            peer_devices.is_disjoint(&selected_devices) && peer_descriptor.id == leader
                        }
                    };

                    if leader_not_selected {
                        setter_leader.set(LeaderSelection::Left(String::from("Select a leader.")));
                    }
                }

                !peer_devices.is_disjoint(&selected_devices)
            })
            .collect::<Vec<_>>()
    });

    let is_leader = move |peer: PeerId| {
        match getter_leader.get() {
            LeaderSelection::Right(leader) => peer == leader,
            LeaderSelection::Left(_) | LeaderSelection::Both(_, _) => false,
        }
    };

    view! {
        <p class="help has-text-danger"> { help_text } </p>
        <div class="table-container mt-2">
            <table class="table is-fullwidth">
                <thead>
                    <tr>
                        <th>Name</th>
                        <th>Peer ID</th>
                        <th>Location</th>
                        <th>Leader</th>
                    </tr>
                </thead>
                <tbody>
                    <For
                        each=move || peers.get()
                        key=|peer| peer.id
                        children=move |peer| {
                            view! {
                                <tr>
                                    <td>
                                        { peer.name.to_string() }
                                    </td>
                                    <td>
                                        { peer.id.to_string() }
                                    </td>
                                    <td>
                                        { peer.location.clone().unwrap_or_default().to_string() }
                                    </td>
                                    <td class="is-narrow" style="text-align: center">
                                        <div class="control">
                                            <label class="radio">
                                                <input
                                                    type = "radio"
                                                    name = "answer"
                                                    checked = is_leader(peer.id)
                                                    on:click = move |_| {
                                                        setter_leader.set(LeaderSelection::Right(peer.id));
                                                    }
                                                />
                                            </label>
                                        </div>
                                    </td>
                                </tr>
                            }
                        }
                    />
                </tbody>
            </table>
        </div>
    }
}
