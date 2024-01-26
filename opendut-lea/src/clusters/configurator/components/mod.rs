mod controls;
mod device_selector;
mod cluster_name_input;
mod leader_selector;

use std::collections::HashSet;
use leptos::{create_local_resource, Resource, Signal, SignalWith};
pub use controls::Controls;
pub use device_selector::{DeviceSelector, DeviceSelection};
pub use cluster_name_input::ClusterNameInput;
pub use leader_selector::LeaderSelector;
use opendut_types::peer::PeerDescriptor;
use opendut_types::topology::DeviceId;
use crate::app::{ExpectGlobals, use_app_globals};


fn get_all_peers() -> Resource<(), Vec<PeerDescriptor>> {
    let globals = use_app_globals();

    create_local_resource(|| {}, move |_| {
        async move {
            let mut carl = globals.expect_client();
            carl.peers.list_peer_descriptors().await
                .expect("Failed to request the list of all peers.")
        }
    })
}

fn get_all_selected_devices(getter: Signal<DeviceSelection>) -> HashSet<DeviceId> {
        getter.with(|selection| match selection {
            DeviceSelection::Left(_) => HashSet::new(),
            DeviceSelection::Right(value) => value.to_owned(),
            DeviceSelection::Both(_, value) => value.to_owned(),
        })
}
