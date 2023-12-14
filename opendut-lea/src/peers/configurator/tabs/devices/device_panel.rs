use leptos::{component, create_slice, event_target_value, IntoView, MaybeSignal, RwSignal, SignalWith, view};

use crate::components::{ButtonColor, ButtonSize, ButtonState, FontAwesomeIcon, IconButton, ReadOnlyInput, SignalToggle, Toggled, UserInput, UserInputValue};
use crate::peers::configurator::types::UserDeviceConfiguration;

#[component]
pub fn DevicePanel(
    device_configuration: RwSignal<UserDeviceConfiguration>,
    is_collapsed: RwSignal<bool>
) -> impl IntoView {
    let device_id = MaybeSignal::derive(move || device_configuration.with(|configuration| configuration.id.to_string()));

    view! {
        <div class="panel is-light">
            <PanelHeading device_configuration is_collapsed />
            <div
                class="panel-block"
                class=("is-hidden", is_collapsed)
            >
                <div class="container">
                    <ReadOnlyInput label="ID" value=device_id />
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
fn PanelHeading(
    device_configuration: RwSignal<UserDeviceConfiguration>,
    is_collapsed: RwSignal<bool>
) -> impl IntoView {

    let collapse_button_icon = is_collapsed.derive_toggled(FontAwesomeIcon::ChevronDown, FontAwesomeIcon::ChevronUp);

    let device_name = move || device_configuration.with(|configuration| {
        match configuration.name {
            UserInputValue::Left(_) => String::from(""),
            UserInputValue::Right(ref value) => value.to_owned(),
            UserInputValue::Both(_, ref value) => value.to_owned()
        }
    });

    view! {
        <div
            class="panel-heading is-clickable px-4 py-3"
            on:click=move |_| is_collapsed.toggle()
        >
            <div class="is-flex is-justify-content-space-between is-align-items-center">
                <span class="is-size-5 has-text-weight-bold">{ device_name }</span>
                <IconButton
                    icon=collapse_button_icon
                    color=ButtonColor::Light
                    size=ButtonSize::Small
                    state=ButtonState::Enabled
                    label="Show Device Details"
                    on_action=move || is_collapsed.toggle()
                />
            </div>
        </div>
    }
}

#[component(transparent)]
fn DeviceNameInput(device_configuration: RwSignal<UserDeviceConfiguration>) -> impl IntoView {

    let (getter, setter) = create_slice(
        device_configuration,
        |configuration| {
            Clone::clone(&configuration.name)
        },
        |configuration, value| {
            configuration.name = value;
        }
    );

    let validator = |input: String| {
        UserInputValue::Right(input)
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
fn DeviceInterfaceInput(device_configuration: RwSignal<UserDeviceConfiguration>) -> impl IntoView {

    let (getter, setter) = create_slice(
        device_configuration,
        |configuration| {
            Clone::clone(&configuration.interface)
        },
        |configuration, value| {
            configuration.interface = value;
        }
    );

    let validator = |input: String| {
        UserInputValue::Right(input)
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
fn DeviceLocationInput(device_configuration: RwSignal<UserDeviceConfiguration>) -> impl IntoView {

    let (getter, setter) = create_slice(
        device_configuration,
        |configuration| {
            Clone::clone(&configuration.location)
        },
        |configuration, value| {
            configuration.location = value;
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
fn DeviceDescriptionInput(device_configuration: RwSignal<UserDeviceConfiguration>) -> impl IntoView {

    let (getter, setter) = create_slice(
        device_configuration,
        |configuration| {
            Clone::clone(&configuration.description)
        },
        |configuration, value| {
            configuration.description = value;
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

