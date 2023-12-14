use leptos::{component, create_read_slice, create_rw_signal, IntoView, RwSignal, SignalWith, view};

use crate::peers::configurator::tabs::devices::device_panel::DevicePanel;
use crate::peers::configurator::types::UserPeerConfiguration;

mod device_panel;

#[component]
pub fn DevicesTab(peer_configuration: RwSignal<UserPeerConfiguration>) -> impl IntoView {
    view! {
        <div>
            <DevicesTable peer_configuration />
        </div>
    }
}

#[component]
fn DevicesTable(peer_configuration: RwSignal<UserPeerConfiguration>) -> impl IntoView {
    let devices = create_read_slice(
        peer_configuration,
        |configuration| {
            Clone::clone(&configuration.devices)
        },
    );

    let panels = move || devices.with(|devices| {
        devices.iter()
            .cloned()
            .map(|device_configuration| {
                let is_collapsed = create_rw_signal(true);
                view! {
                    <DevicePanel device_configuration is_collapsed />
                }
            })
            .collect::<Vec<_>>()
    });

    view! {
        <div>
            { panels }
        </div>
    }
}
