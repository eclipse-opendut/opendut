use leptos::{component, create_memo, create_read_slice, create_rw_signal, IntoView, RwSignal, SignalGet, SignalUpdate, SignalWith, view};

use opendut_types::topology::DeviceId;

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
    let devices = create_read_slice(
        peer_configuration,
        |configuration| {
            Clone::clone(&configuration.devices)
        },
    );

    let panels = create_memo(move |_| {
        devices.with(|devices| {
            devices.iter()
                .cloned()
                .map(|device_configuration| {
                    view! {
                        <DevicePanel device_configuration />
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
