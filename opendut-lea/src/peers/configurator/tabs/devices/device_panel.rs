use leptos::either::Either;
use leptos::prelude::*;
use opendut_model::topology::{DeviceDescription, DeviceId, DeviceName, IllegalDeviceName};
use opendut_model::util::net::NetworkInterfaceId;
use uuid::Uuid;
use opendut_lea_components::{SelectionOption, UserSelect};
use crate::components::{ButtonColor, ButtonSize, ButtonState, ConfirmationButton, DoorhangerButton, FontAwesomeIcon, IconButton, ReadOnlyInput, Toggled, UserInput, UserInputValue, UserTextarea};
use crate::peers::configurator::types::devices::UserDeviceConfiguration;
use crate::peers::configurator::types::{UserPeerConfiguration, EMPTY_DEVICE_NAME_ERROR_MESSAGE};
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
                UserInputValue::Left(_) => String::new(),
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
                    on_confirm={
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
            placeholder="Device_A"
            validator
        />
    }
}

#[component]
fn DeviceInterfaceInput(
    peer_configuration: RwSignal<UserPeerConfiguration>,
    device_configuration: RwSignal<UserDeviceConfiguration>,
) -> impl IntoView {
    const INITIAL_OPTION: &str = "Select interface";

    let peer_network_interfaces = create_read_slice(
        peer_configuration,
        |peer_configuration| {
            Clone::clone(&peer_configuration.network.network_interfaces)
        },
    );

    let (getter, setter) = create_slice(
        device_configuration,
        |device_configuration| {
            match device_configuration.interface.as_ref() {
                Some(interface_id) => {
                    UserInputValue::Right(interface_id.to_string())
                }
                None => {
                    UserInputValue::Left(INITIAL_OPTION.to_owned())
                }
            }
        },
        |device_configuration, input| {
            device_configuration.interface = match input {
                UserInputValue::Right(value)
                | UserInputValue::Both(_, value) => {
                    let uuid = Uuid::parse_str(&value).expect(
                        "Should be a valid UUID, which we passed in as option-value.",
                    );

                    Some(NetworkInterfaceId::from(uuid))
                }
                UserInputValue::Left(_) => None,
            };
        },
    );

    let options = Signal::derive(move || {
        peer_network_interfaces.with(|interfaces| {
            interfaces.iter()
                .map(|interface| {
                    let interface = interface.get_untracked();

                    SelectionOption {
                        display_name: format!(
                            "{} ({})",
                            interface.name.name(),
                            interface.configuration.display_name(),
                        ),
                        value: interface.id.to_string(),
                    }
                })
                .collect()
        })
    });

    let initial_option = INITIAL_OPTION.to_owned();

    view! {
        <UserSelect
            options
            initial_option
            getter
            setter
            label="Interface"
        />
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
