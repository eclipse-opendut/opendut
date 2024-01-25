use std::collections::HashSet;
use std::ops::Not;
use leptos::*;
use opendut_types::peer::PeerId;

use opendut_types::topology::{Device, DeviceId};
use crate::app::{ExpectGlobals, use_app_globals};

use crate::clusters::configurator::types::UserClusterConfiguration;
use crate::components::{ButtonColor, ButtonSize, ButtonState, FontAwesomeIcon, IconButton};
use crate::util::{Ior, NON_BREAKING_SPACE};

pub type DeviceSelectionError = String;
pub type DeviceSelection = Ior<DeviceSelectionError, HashSet<DeviceId>>;

#[component]
pub fn DeviceSelector(cluster_configuration: RwSignal<UserClusterConfiguration>) -> impl IntoView {
    let globals = use_app_globals();

    let peer_descriptors = create_local_resource(|| {}, move |_| {
        async move {
            let mut carl = globals.expect_client();
            carl.peers.list_peer_descriptors().await
                .expect("Failed to request the list of devices.")
        }
    });

    let (getter, setter) = create_slice(cluster_configuration,
                                        |config| {
                                            Clone::clone(&config.devices)
                                        },
                                        |config, input| {
                                            config.devices = input;
                                        }
    );

    let selected_devices = move || {
        getter.with(|selection| match selection {
            DeviceSelection::Left(_) => HashSet::new(),
            DeviceSelection::Right(value) => value.to_owned(),
            DeviceSelection::Both(_, value) => value.to_owned(),
        })
    };

    let help_text = move || {
        getter.with(|selection| match selection {
            DeviceSelection::Right(_) => String::from(NON_BREAKING_SPACE),
            DeviceSelection::Left(error) => error.to_owned(),
            DeviceSelection::Both(error, _) => error.to_owned(),
        })
    };

    let rows = move || {

        let mut all_devices_by_peer: Vec<_> = Vec::new();

        for peer in peer_descriptors.get().unwrap_or_default() {
            let mut devices = peer.topology.devices;
            // let mut devices = devices.get().unwrap_or_default();
            let selected_devices = selected_devices();

            devices.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

            let devices_per_peer = devices.clone().into_iter()
                .map(|device| {
                    let collapsed_signal = create_rw_signal(true);
                    let selected_signal = create_rw_signal(selected_devices.contains(&device.id));
                    view! {
                    <tr>
                        <td class="is-narrow">
                            <IconButton
                                icon=FontAwesomeIcon::ChevronDown
                                color=ButtonColor::White
                                size=ButtonSize::Small
                                state=ButtonState::Enabled
                                label="More infos"
                                on_action=move || collapsed_signal.update(|collapsed| *collapsed = collapsed.not())
                            />
                        </td>
                        <td>
                            {&device.name}
                        </td>
                        <td>
                            {&device.location}
                        </td>
                        <td class="is-narrow">
                                <IconButton
                                    icon=FontAwesomeIcon::Check
                                    color=MaybeSignal::derive(move || match selected_signal.get() {
                                        false => ButtonColor::Light,
                                        true => ButtonColor::Success,
                                    })
                                    size=ButtonSize::Small
                                    state=ButtonState::Enabled
                                    label="More infos"
                                    on_action=move || icon_button_on_action(
                                            selected_signal,
                                            getter,
                                            setter,
                                            device.id,
                                        )
                            />
                        </td>
                    </tr>
                    <tr hidden={collapsed_signal}>
                        <DeviceInfo
                            device = device
                            peer_id = peer.id
                        />
                    </tr>
                }
                })
                .collect::<Vec<_>>();

            for device in devices_per_peer {
                all_devices_by_peer.push(device);
            }
        }
        all_devices_by_peer
    };

    view! {
        <p class="help has-text-danger" align="right">{ help_text }</p>
        <div class="table-container">
            <table class="table is-narrow is-fullwidth">
                <thead>
                    <tr>
                        <th></th>
                        <th>Name</th>
                        <th>Location</th>
                        <th></th>
                    </tr>
                </thead>
                    <tbody>
                        { rows }
                    </tbody>
            </table>
        </div>
    }
}

#[component]
pub fn DeviceInfo(
    device: Device,
    peer_id: PeerId,
) -> impl IntoView {
    view! {
        <td></td>
        <td colspan="3">
            <div class="field" style="margin-bottom: 0;">
                <label class="label">ID</label>
                <div class="control">
                    <p>{device.id.to_string()}</p>
                </div>
            </div>
            <hr width="30%" style="margin-bottom: 0; margin-top: 0;"/>
            <div class="field" style="margin-bottom: 0;">
                <label class="label">Peer ID</label>
                <div class="control">
                    <p>{peer_id.to_string()}</p>
                </div>
            </div>
            <hr width="30%" style="margin-bottom: 0; margin-top: 0;"/>
            <div class="field" style="margin-bottom: 0;">
                <label class="label">Interface</label>
                <div class="control">
                    <p>{device.interface.name()}</p>
                </div>
            </div>
            <hr width="30%" style="margin-bottom: 0; margin-top: 0;"/>
            <div class="field" style="margin-bottom: 0;">
                <label class="label">Tags</label>
                <div class="control">
                    <p>{device.tags}</p>
                </div>
            </div>
            <hr width="30%" style="margin-bottom: 0; margin-top: 0;"/>
            <div class="field">
                <label class="label">Description</label>
                <div class="control">
                    <p>{device.description}</p>
                </div>
            </div>
        </td>
    }
}

pub fn icon_button_on_action(
    selected_signal: RwSignal<bool>,
    getter: Signal<DeviceSelection>,
    setter: SignalSetter<DeviceSelection>,
    device_id: DeviceId,
) {
    selected_signal.update(|selected| *selected = selected.not());
    let insert = selected_signal.get();
    let device_selection = match getter.get() {
        DeviceSelection::Left(error) => {
            if insert {
                Ior::Both(String::from("Select at least one more device."), HashSet::from([device_id]))
            } else {
                Ior::Left(error)
            }
        }
        DeviceSelection::Right(mut devices) | DeviceSelection::Both(_, mut devices) => {
            if insert {
                devices.insert(device_id);
            } else {
                devices.remove(&device_id);
            }
            match devices.len() {
                0 => {Ior::Left(String::from("Select at least two devices."))}
                1 => {Ior::Both(String::from("Select at least one more device."), devices)}
                _ => {Ior::Right(devices)}
            }
        }
    };
    setter.set(device_selection);
}