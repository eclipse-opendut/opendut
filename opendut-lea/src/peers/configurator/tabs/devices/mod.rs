use leptos::{component, create_memo, create_rw_signal, create_slice, IntoView, RwSignal, SignalGetUntracked, SignalUpdate, SignalWith, SignalWithUntracked, view};

use opendut_types::topology::{Device, DeviceId};

use crate::components::UserInputValue;
use crate::peers::configurator::tabs::devices::device_panel::DevicePanel;
use crate::peers::configurator::types::{EMPTY_DEVICE_INTERFACE_ERROR_MESSAGE, EMPTY_DEVICE_NAME_ERROR_MESSAGE, UserDeviceConfiguration, UserPeerConfiguration};

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

    let (devices, devices_setter) = create_slice(peer_configuration,
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

    let panels = create_memo(move |_| {
        devices.with(|devices| {
            devices.iter()
                .cloned()
                .map(|device_configuration| {
                    view! {
                        <DevicePanel device_configuration on_delete=on_device_delete />
                    }
                })
                .collect::<Vec<_>>()
        })
    });

    view! {
        <div>
            <div>
                { panels }
            </div>
             <div class="mt-5">
                <div
                    class="dut-panel-ghost has-text-success px-4 py-3 is-clickable is-flex is-justify-content-center"
                    on:click=move |_| {
                        peer_configuration.update(|peer_configuration| {
                            let device_id = DeviceId::random();
                            let user_device_configuration = create_rw_signal(
                                UserDeviceConfiguration {
                                    id: device_id,
                                    name: UserInputValue::Left(String::from(EMPTY_DEVICE_NAME_ERROR_MESSAGE)),
                                    description: String::new(),
                                    location: UserInputValue::Right(String::new()),
                                    interface: UserInputValue::Left(String::from(EMPTY_DEVICE_INTERFACE_ERROR_MESSAGE)),
                                    is_collapsed: false
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
