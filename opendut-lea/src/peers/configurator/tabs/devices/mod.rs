use leptos::prelude::*;
use opendut_model::topology::DeviceId;

use crate::components::UserInputValue;
use crate::peers::configurator::tabs::devices::device_panel::DevicePanel;
use crate::peers::configurator::types::devices::UserDeviceConfiguration;
use crate::peers::configurator::types::{UserPeerDescriptor, EMPTY_DEVICE_NAME_ERROR_MESSAGE};

mod device_panel;
mod name_input;
mod interface_input;
mod description_input;
mod tag_input;

#[component]
pub fn DevicesTab(user_peer_descriptor: RwSignal<UserPeerDescriptor>) -> impl IntoView {
    view! {
        <div>
            <DevicesTable user_peer_descriptor />
        </div>
    }
}

#[component]
fn DevicesTable(user_peer_descriptor: RwSignal<UserPeerDescriptor>) -> impl IntoView {

    let (devices, devices_setter) = create_slice(user_peer_descriptor,
        |peer_configuration| {
            Clone::clone(&peer_configuration.devices)
        },
        |peer_configuration, value| {
            peer_configuration.devices = value
        }
    );

    let on_device_delete = move |device_id: DeviceId| {
        let remaining_devices = devices.with_untracked(|devices| {
            devices.iter()
                .filter(|device| device.with_untracked(|device| device_id != device.id))
                .cloned()
                .collect::<Vec<_>>()
        });
        devices_setter.set(remaining_devices)
    };

    let panels = move || {
        devices.with(|devices| {
            devices.iter()
                .cloned()
                .map(|device_configuration| {
                    view! {
                        <DevicePanel user_peer_descriptor device_configuration on_delete=on_device_delete />
                    }
                })
                .collect::<Vec<_>>()
        })
    };

    view! {
        <div>
            <div>
                { panels }
            </div>
             <div class="mt-5">
                <div
                    class="dut-panel-ghost has-text-success px-4 py-3 is-clickable is-flex is-justify-content-center"
                    on:click=move |_| {
                        user_peer_descriptor.update(|peer_configuration| {
                            let device_id = DeviceId::random();
                            let user_device_configuration = RwSignal::new(
                                UserDeviceConfiguration {
                                    id: device_id,
                                    name: UserInputValue::Left(String::from(EMPTY_DEVICE_NAME_ERROR_MESSAGE)),
                                    description: UserInputValue::Right(String::from("")),
                                    interface: None,
                                    is_collapsed: false,
                                    contained_in_clusters: vec![],
                                    tags: vec![],
                                }
                            );
                            peer_configuration.devices.push(user_device_configuration);
                        });
                    }
                >
                    <span><i class="fa-solid fa-circle-plus"></i></span>
                </div>
            </div>
        </div>
    }
}
