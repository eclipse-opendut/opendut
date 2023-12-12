use std::collections::HashSet;
use leptos::*;

use opendut_types::topology::{Device, DeviceId};

use crate::api::use_carl;
use crate::clusters::configurator::types::UserClusterConfiguration;
use crate::util::{Ior, NON_BREAKING_SPACE};

pub type DeviceSelectionError = String;
pub type DeviceSelection = Ior<DeviceSelectionError, HashSet<DeviceId>>;

#[component]
pub fn DeviceSelector(cluster_configuration: RwSignal<UserClusterConfiguration>) -> impl IntoView {

    let carl = use_carl();

    let getter = create_read_slice(cluster_configuration,
        |config| {
            Clone::clone(&config.devices)
        },
    );

    let devices: Resource<(), Vec<Device>> = create_local_resource(|| {}, move |_| {
        async move {
            let mut carl = carl.get_untracked();
            carl.peers.list_devices().await
                .expect("Failed to request the list of devices.")
        }
    });

    let help_text = move || {
        getter.with(|selection| match selection {
            DeviceSelection::Right(_) => String::from(NON_BREAKING_SPACE),
            DeviceSelection::Left(error) => error.to_owned(),
            DeviceSelection::Both(error, _) => error.to_owned(),
        })
    };

    let table = move || {
        devices.with(|devices| devices.as_ref().map(|devices| {
            if devices.is_empty() {
                view! {
                    <div class="notification is-warning is-light">
                        <p>"No devices available."</p>
                    </div>
                }
            }
            else {
                let rows = devices.iter().map(|device| {
                    let device_id = device.id;
                    view! {
                        <Row
                            name={ Clone::clone(&device.name) }
                            description={ Clone::clone(&device.description) }
                            tags={ Clone::clone(&device.tags) }
                            on_change=move |selected| {
                                cluster_configuration.update(|configuration| {
                                    match configuration.devices.as_mut() {
                                        Ior::Left(_) => {
                                            if selected {
                                                configuration.devices = Ior::Both(String::from("Select at least one more device."), HashSet::from([device_id]));
                                            }
                                        },
                                        Ior::Right(devices) | Ior::Both(_, devices) => {
                                            if selected {
                                                devices.insert(device_id);
                                            }
                                            else {
                                                devices.remove(&device_id);
                                            }
                                            match devices.len() {
                                                0 => configuration.devices = Ior::Left(String::from("Select at least two devices.")),
                                                1 => configuration.devices = Ior::Both(String::from("Select at least one more device."), Clone::clone(devices)),
                                                _ => configuration.devices = Ior::Right(Clone::clone(devices)),
                                            }
                                        },
                                    }
                                });
                            }
                        />
                    }
                }).collect::<Vec<_>>();

                view! {
                    <div>
                        <p class="help has-text-danger">{ help_text }</p>
                        <div class="control">
                            <table class="table is-hoverable is-fullwidth">
                                <thead>
                                    <tr>
                                        <th></th>
                                        <th>"Name"</th>
                                        <th>"Description"</th>
                                        <th>"Tags"</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    { rows }
                                </tbody>
                            </table>
                        </div>
                    </div>
                }
            }
        }))
    };

    view! {
        <div class="field pt-2">
            <label class="label">"Devices"</label>
            <Suspense fallback=move || view! { <p>"Loading..."</p> }>
                { table }
            </Suspense>
        </div>
    }
}

#[component]
fn Row<F>(
    name: String,
    description: String,
    tags: Vec<String>,
    on_change: F,
) -> impl IntoView
where F: Fn(bool) + 'static {

    let tags = move || {
        tags.iter()
            .map(|tag| {
                view! {
                    <span class="tag is-info is-light">{ tag }</span>
                }
            })
            .collect::<Vec<_>>()
    };

    let (is_selected, set_selected) = create_signal(false);

    let checkbox_aria_label = {
        let name = Clone::clone(&name);
        move || {
            if is_selected.get() {
                format!("Deselect device {}", name)
            }
            else {
                format!("Select device {}", name)
            }
        }
    };

    create_effect(move |_| {
        on_change(is_selected.get());
    });

    view! {
            <tr
                class="is-clickable"
                on:click={
                    move |_| {
                        set_selected.update(|value| *value = !*value);
                    }
                }
            >
                <td>
                    <input
                        type="checkbox"
                        checked=move || { is_selected.get() }
                        aria-label={ checkbox_aria_label }
                        on:input=move |ev| {
                            set_selected.update(|value| *value = event_target_checked(&ev));
                        }
                    />
                </td>
                <td>{ name }</td>
                <td>{ description }</td>
                <td><div class="tags">{ tags }</div></td>
            </tr>
        }
}
