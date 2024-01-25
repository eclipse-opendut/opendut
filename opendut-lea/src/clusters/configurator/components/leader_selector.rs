use std::collections::{HashMap, HashSet};
use leptos::{component, create_local_resource, create_rw_signal, create_slice, IntoView, MaybeSignal, RwSignal, Signal, SignalGet, SignalSetter, SignalUpdate, SignalWith, view};
use tracing::info;
use opendut_types::peer::PeerId;
use opendut_types::topology::{Device, DeviceId};
use crate::app::{ExpectGlobals, use_app_globals};
use crate::clusters::configurator::components::DeviceSelection;
use crate::clusters::configurator::types::UserClusterConfiguration;

pub type LeaderSelection = HashMap<PeerId, HashSet<DeviceId>>;
#[component]
pub fn LeaderSelector(cluster_configuration: RwSignal<UserClusterConfiguration>) -> impl IntoView {
    let globals = use_app_globals();

    let peer_descriptors = create_local_resource(|| {}, move |_| {
        async move {
            let mut carl = globals.expect_client();
            carl.peers.list_peer_descriptors().await
                .expect("Failed to request the list of devices.")
        }
    });

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

    let selected_devices = move || {
        getter_selected_devices.with(|selection| match selection {
            DeviceSelection::Left(_) => HashSet::new(),
            DeviceSelection::Right(value) => value.to_owned(),
            DeviceSelection::Both(_, value) => value.to_owned(),
        })
    };

    let rows = move || {

        let selected_devices = selected_devices();

        let mut peers = peer_descriptors.get().unwrap_or_default();

        peers.sort_by(|a, b| a.name.to_string().to_lowercase().cmp(&b.name.to_string().to_lowercase()));

        peers.clone().into_iter()
            .filter( |x| {
                let mut topology: HashSet<DeviceId> = HashSet::new();
                for y in &x.topology.devices {
                    topology.insert(y.id);
                }
                !topology.is_disjoint(&selected_devices)
            })
            .map(|peer| {
                info!("PEER ID: {:?}", peer.id.to_string());
                info!("GETTER ID: {:?}", getter_leader.get().to_string());
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
            <p class="help has-text-info" align="left">Wird hier kein Leader spezifiziert, wird beim Deployment einer automatisch gewaehlt.</p>
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