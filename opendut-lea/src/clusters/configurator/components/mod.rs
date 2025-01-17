mod controls;
mod device_selector;
mod cluster_name_input;
mod leader_selector;

use std::collections::HashSet;
use leptos::prelude::*;
pub use controls::Controls;
pub use device_selector::{DeviceSelector, DeviceSelection};
pub use cluster_name_input::ClusterNameInput;
pub use leader_selector::{LeaderSelector, LeaderSelection};
use opendut_types::topology::DeviceId;


fn get_all_selected_devices(getter: Signal<DeviceSelection>) -> HashSet<DeviceId> {
    getter.with(|selection| match selection {
        DeviceSelection::Left(_) => HashSet::new(),
        DeviceSelection::Right(value) => value.to_owned(),
        DeviceSelection::Both(_, value) => value.to_owned(),
    })
}
