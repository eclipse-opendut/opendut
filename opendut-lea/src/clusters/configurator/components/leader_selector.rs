use std::collections::{HashMap, HashSet};
use leptos::{component, create_slice, IntoView, RwSignal, SignalGet, view};
use opendut_types::peer::PeerId;
use opendut_types::topology::DeviceId;
use crate::app::use_app_globals;
use crate::clusters::configurator::components::{get_all_peers, get_all_selected_devices};
use crate::clusters::configurator::types::UserClusterConfiguration;

pub type LeaderSelection = HashMap<PeerId, HashSet<DeviceId>>;
#[component]
pub fn LeaderSelector(cluster_configuration: RwSignal<UserClusterConfiguration>) -> impl IntoView {
    let globals = use_app_globals();

    let peer_descriptors = get_all_peers();

    let (getter_selected_devices, setter_selected_devices) = create_slice(cluster_configuration,
                                        |config| {
                                            Clone::clone(&config.devices)
                                        },
                                        |config, input| {
                                            config.devices = input;
                                        }
    );

    let (getter_leader, setter_leader) = create_slice(cluster_configuration,
                                        |config| {
                                            Clone::clone(&config.leader)
                                        },
                                        |config, input| {
                                            config.leader = input;
                                        }
    );

    let selected_devices = move || { get_all_selected_devices(getter_selected_devices) };

    let rows = move || {

        let selected_devices = selected_devices();

        let mut peers = peer_descriptors.get().unwrap_or_default();

        peers.sort_by(|a, b| a.name.to_string().to_lowercase().cmp(&b.name.to_string().to_lowercase()));

        peers.clone().into_iter()
            .filter( |peer_descriptor| {
                let mut peer_devices: HashSet<DeviceId> = HashSet::new();
                for device in &peer_descriptor.topology.devices {
                    peer_devices.insert(device.id);
                }
                if peer_devices.is_disjoint(&selected_devices) &&
                    peer_descriptor.id.to_string().eq(&getter_leader.get().to_string())
                {
                    setter_leader.set(PeerId::default());
                }
                !peer_devices.is_disjoint(&selected_devices)
            })
            .map(|peer| {
                view! {
                <tr>
                    <td>
                        {&peer.name.to_string()}
                    </td>
                    <td>
                        {&peer.id.to_string()}
                    </td>
                    <td class="is-narrow" style="text-align: center">
                        <div class="control">
                            <label class="radio">
                                <input
                                    type = "radio"
                                    name = "answer"
                                    checked = move || {
                                            peer.id.to_string().eq(&getter_leader.get().to_string())
                                    }
                                    on:click = move |_| {
                                        setter_leader.set(peer.id);
                                    }
                                />
                            </label>
                        </div>
                    </td>
                </tr>
            }
            })
            .collect::<Vec<_>>()

    };

    view! {
        <div>
            <p class="help has-text-info" align="left">If no leader is specified here, one is automatically selected during deployment.</p>
        </div>
        <div class="table-container">
            <table class="table is-narrow is-fullwidth">
                <thead>
                    <tr>
                        <th>Name</th>
                        <th>Peer ID</th>
                        <th>Leader</th>
                    </tr>
                </thead>
                    <tbody>
                        { rows }
                    </tbody>
            </table>
        </div>
    }
}