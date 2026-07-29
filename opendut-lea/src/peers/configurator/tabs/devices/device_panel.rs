use leptos::either::Either;
use leptos::prelude::*;
use opendut_model::topology::DeviceId;

use crate::components::{ButtonColor, ButtonSize, ButtonState, ConfirmationButton, DoorhangerButton, FontAwesomeIcon, IconButton, ReadOnlyInput, Toggled, UserInputValue};
use crate::peers::configurator::tabs::devices::description_input::DeviceDescriptionInput;
use crate::peers::configurator::tabs::devices::interface_input::DeviceInterfaceInput;
use crate::peers::configurator::tabs::devices::name_input::DeviceNameInput;
use crate::peers::configurator::tabs::devices::tag_input::DeviceTagInput;
use crate::peers::configurator::types::devices::UserDeviceConfiguration;
use crate::peers::configurator::types::UserPeerDescriptor;
use crate::routing;

#[component]
pub fn DevicePanel<OnDeleteFn>(
    user_peer_descriptor: RwSignal<UserPeerDescriptor>,
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
                    <DeviceInterfaceInput user_peer_descriptor device_configuration />
                    <DeviceInterfaceInput user_peer_descriptor device_configuration />
                    <DeviceTagInput device_configuration />
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
