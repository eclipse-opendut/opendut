use leptos::either::Either;
use leptos::prelude::*;
use opendut_model::topology::{DeviceDescription, DeviceId, DeviceName, IllegalDeviceName};
use opendut_model::util::net::NetworkInterfaceId;
use uuid::Uuid;
use crate::components::{ButtonColor, ButtonSize, ButtonState, ConfirmationButton, DoorhangerButton, FontAwesomeIcon, IconButton, ReadOnlyInput, Toggled, UserInput, UserInputValue, UserTextarea};
use crate::peers::configurator::types::{EMPTY_DEVICE_NAME_ERROR_MESSAGE, UserDeviceConfiguration, UserPeerConfiguration};
use crate::routing;

#[component]
pub fn DevicePanel<OnDeleteFn>(
    peer_configuration: RwSignal<UserPeerConfiguration>,
    device_configuration: RwSignal<UserDeviceConfiguration>,
    on_delete: OnDeleteFn
) -> impl IntoView
where
    OnDeleteFn: Fn(DeviceId) + 'static + Copy + Send + Sync
{
    let device_id_string = Signal::derive(move || device_configuration.get().id.to_string());
    let is_collapsed = move || device_configuration.get().is_collapsed;

    view! {
        <div class="panel is-light">
            <DevicePanelHeading device_configuration on_delete />
            <div
                class="panel-block"
                class=("is-hidden", is_collapsed)
            >
                <div class="container">
                    <ReadOnlyInput label="ID" value=device_id_string />
                    <DeviceNameInput device_configuration />
                    <DeviceInterfaceInput peer_configuration device_configuration />
                    <DeviceDescriptionInput device_configuration />
                </div>
            </div>
        </div>
    }
}

#[component]
fn DevicePanelHeading<OnDeleteFn>(
    device_configuration: RwSignal<UserDeviceConfiguration>,
    on_delete: OnDeleteFn
) -> impl IntoView
where
    OnDeleteFn: Fn(DeviceId) + 'static + Copy + Send + Sync
{
    let (is_collapsed, set_is_collapsed) = create_slice(device_configuration,
        move |device_configuration| {
            device_configuration.is_collapsed
        },
        move |device_configuration, value| {
            device_configuration.is_collapsed = value;
        }
    );

    let collapse_button_icon = is_collapsed.derive_toggled(FontAwesomeIcon::ChevronDown, FontAwesomeIcon::ChevronUp);

    let device_name = create_read_slice(device_configuration,
        |device_configuration| {
            match device_configuration.name {
                UserInputValue::Left(_) => String::from(""),
                UserInputValue::Right(ref value) => value.to_owned(),
                UserInputValue::Both(_, ref value) => value.to_owned()
            }
        }
    );

    let delete_button = move || {
        let used_clusters = device_configuration.get().contained_in_clusters.len();

        if used_clusters > 0 {
            Either::Left(view! {
                <DoorhangerButton
                    icon=FontAwesomeIcon::TrashCan
                    color=ButtonColor::Light
                    size=ButtonSize::Small
                    state=ButtonState::Enabled
                    label="Delete Device?"
                >
                    <div style="white-space: nowrap">
                        "Device can not be removed while it is configured in "{used_clusters}
                        <a class="has-text-link" href=routing::path::clusters_overview>" cluster(s)"</a>
                    </div>
                </DoorhangerButton>
            })
        } else {
            Either::Right(view! {
                <ConfirmationButton
                    icon=FontAwesomeIcon::TrashCan
                    color=ButtonColor::Light
                    size=ButtonSize::Small
                    state=ButtonState::Enabled
                    label="Delete Device?"
                    on_conform={
                        move || on_delete(device_configuration.get_untracked().id)
                    }
                />
            })
        }
    };

    view! {
        <div class="panel-heading px-2 py-3">
            <div class="is-flex is-justify-content-space-between is-align-items-center">
                <div>
                    <span class="pr-1">
                        <IconButton
                            icon=collapse_button_icon
                            color=ButtonColor::Light
                            size=ButtonSize::Small
                            state=ButtonState::Enabled
                            label="Show Device Details"
                            on_action=move || set_is_collapsed.set(!is_collapsed.get_untracked())
                        />
                    </span>
                    <span class="is-size-5 has-text-weight-bold">{ device_name }</span>
                </div>
                <div>
                    { delete_button }
                </div>
            </div>
        </div>
    }
}

#[component]
fn DeviceNameInput(
    device_configuration: RwSignal<UserDeviceConfiguration>,
) -> impl IntoView {

    let (getter, setter) = create_slice(device_configuration,
        |device_configuration| {
            Clone::clone(&device_configuration.name)
        },
        |device_configuration, value| {
            device_configuration.name = value;
        }
    );

    let validator = |input: String| {
        match DeviceName::try_from(input.clone()) {
            Ok(_) => {
                UserInputValue::Right(input)
            }
            Err(cause) => {
                match cause {
                    IllegalDeviceName::TooShort { expected, actual, value } => {
                        if actual > 0 {
                            UserInputValue::Both(format!("A device name must be at least {expected} characters long."), value)

                        }
                        else {
                            UserInputValue::Both(String::from(EMPTY_DEVICE_NAME_ERROR_MESSAGE), value)
                        }
                    },
                    IllegalDeviceName::TooLong { expected, value, .. } => {
                        UserInputValue::Both(format!("A device name must be at most {expected} characters long."), value)
                    },
                    IllegalDeviceName::InvalidStartEndCharacter { value } => {
                        UserInputValue::Both("The device name starts/ends with an invalid character. \
                        Valid characters are a-z, A-Z and 0-9.".to_string(), value)
                    }
                    IllegalDeviceName::InvalidCharacter { value } => {
                        UserInputValue::Both("The device name contains invalid characters. \
                        Valid characters are a-z, A-Z, 0-9 and _-".to_string(), value)
                    }
                }
            }
        }
    };

    view! {
        <UserInput
            getter
            setter
            label="Name"
            placeholder="Device A"
            validator
        />
    }
}

#[component]
fn DeviceInterfaceInput(
    peer_configuration: RwSignal<UserPeerConfiguration>,
    device_configuration: RwSignal<UserDeviceConfiguration>,
) -> impl IntoView {

    let peer_network_interfaces = create_read_slice(peer_configuration,
        |peer_network_interfaces| {
            Clone::clone(&peer_network_interfaces.network.network_interfaces)
        }
    );

    let (device_interface_id, set_device_interface_id) = create_slice(device_configuration,
        |device_configuration| {
            Clone::clone(&device_configuration.interface)
        },
        |device_configuration, value| {
            device_configuration.interface = value;
        }
    );

    let dropdown_options = move || {
        peer_network_interfaces.with(|interfaces | {
            interfaces.iter().cloned()
                .map(|interface| {
                    let id = interface.get_untracked().id;
                    let name = interface.get_untracked().name.name();
                    let interface_type = interface.get_untracked().configuration.display_name();

                    if device_interface_id.get_untracked() == Some(interface.get_untracked().id) {
                        Either::Left(view! {
                            <option value={id.to_string()} selected>{name} " ("{interface_type}")"</option>
                        })
                    } else {
                        Either::Right(view! {
                            <option value={id.to_string()}>{name} " ("{interface_type}")"</option>
                        })
                    }
                })
                .collect::<Vec<_>>()
        })
    };

    let parse_selected_interface_id = move |selected_interface_id: String| {
        NetworkInterfaceId::from(
            Uuid::parse_str(&selected_interface_id)
                .expect("Should be a valid UUID, which we passed in as option-value.")
        )
    };

    view! {
        <div class="field">
            <label class="label">Interface</label>
            <div class="control">
                <div class="select"
                    on:change=move |ev| {
                        let target_value = event_target_value(&ev);
                        if target_value == "Select interface" {
                            set_device_interface_id.set(None);
                        } else {
                            set_device_interface_id.set(
                                Some(parse_selected_interface_id(target_value))
                            );
                        }
                    }>
                    <select>
                        <option>Select interface</option>
                        { dropdown_options }
                    </select>
                </div>
            </div>
        </div>
    }
}

#[component]
fn DeviceDescriptionInput(
    device_configuration: RwSignal<UserDeviceConfiguration>
) -> impl IntoView {

    let (getter, setter) = create_slice(device_configuration,
        |device_configuration| {
            Clone::clone(&device_configuration.description)
        },
        |device_configuration, value| {
            device_configuration.description = value;
        }
    );

    let validator = |input: String| {
        match DeviceDescription::try_from(Clone::clone(&input)) {
            Err(error) => {
                UserInputValue::Both(error.to_string(), input)
            }
            Ok(_) => {
                UserInputValue::Right(input)
            }
        }
    };

    view! {
        <UserTextarea
            getter=getter
            setter=setter
            label="Description"
            placeholder="Description"
            validator
        />
    }
}

