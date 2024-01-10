use leptos::{component, create_read_slice, create_slice, event_target_value, IntoView, MaybeSignal, RwSignal, SignalGet, SignalGetUntracked, view};
use opendut_types::topology::DeviceId;
use opendut_types::util::net::NetworkInterfaceName;

use crate::components::{ButtonColor, ButtonSize, ButtonState, ConfirmationButton, FontAwesomeIcon, IconButton, ReadOnlyInput, Toggled, UserInput, UserInputValue};
use crate::peers::configurator::types::{EMPTY_DEVICE_NAME_ERROR_MESSAGE, UserDeviceConfiguration};

#[component]
pub fn DevicePanel<OnDeleteFn>(
    device_configuration: RwSignal<UserDeviceConfiguration>,
    on_delete: OnDeleteFn
) -> impl IntoView
where
    OnDeleteFn: Fn(DeviceId) + 'static
{
    let device_id_string = MaybeSignal::derive(move || device_configuration.get().id.to_string());
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
                    <DeviceInterfaceInput device_configuration />
                    <DeviceLocationInput device_configuration />
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
    OnDeleteFn: Fn(DeviceId) + 'static
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

    view! {
        <div
            class="panel-heading px-2 py-3"
            // on:click=move |_| set_is_collapsed.set(!is_collapsed.get_untracked()) // Leads to problems with the door-hanger, because it tries to collapse a deleted device.
        >
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
                    <ConfirmationButton
                        icon=FontAwesomeIcon::TrashCan
                        color=ButtonColor::Light
                        size=ButtonSize::Small
                        state=ButtonState::Enabled
                        label="Delete Device?"
                        on_conform=move || on_delete(device_configuration.get_untracked().id)
                    />
                </div>
            </div>
        </div>
    }
}

#[component(transparent)]
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
        if input.is_empty() {
            UserInputValue::Left(String::from(EMPTY_DEVICE_NAME_ERROR_MESSAGE))
        }
        else {
            UserInputValue::Right(input)
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

#[component(transparent)]
fn DeviceInterfaceInput(
    device_configuration: RwSignal<UserDeviceConfiguration>,
) -> impl IntoView {

    let (getter, setter) = create_slice(device_configuration,
        |device_configuration| {
            Clone::clone(&device_configuration.interface)
        },
        |device_configuration, value| {
            device_configuration.interface = value;
        }
    );

    let validator = |input: String| {
        match NetworkInterfaceName::try_from(Clone::clone(&input)) {
            Err(error) => {
                UserInputValue::Both(error.to_string(), input)
            }
            Ok(_) => {
                UserInputValue::Right(input)
            }
        }

    };

    view! {
        <UserInput
            getter=getter
            setter=setter
            label="Interface"
            placeholder="eth0"
            validator
        />
    }
}

#[component(transparent)]
fn DeviceLocationInput(
    device_configuration: RwSignal<UserDeviceConfiguration>,
) -> impl IntoView {

    let (getter, setter) = create_slice(device_configuration,
        |device_configuration| {
            Clone::clone(&device_configuration.location)
        },
        |device_configuration, value| {
            device_configuration.location = value;
        }
    );

    let validator = |input: String| {
        UserInputValue::Right(input)
    };

    view! {
        <UserInput
            getter=getter
            setter=setter
            label="Location"
            placeholder="Ulm, Germany"
            validator
        />
    }
}

#[component(transparent)]
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

    view! {
        <div class="field">
            <label class="label">"Description"</label>
            <div class="control">
                <textarea
                    class="textarea"
                    aria-label="Description"
                    prop:value=getter
                    on:input=move |event| {
                        setter.set(event_target_value(&event));
                    }
                />
            </div>
        </div>
    }
}

