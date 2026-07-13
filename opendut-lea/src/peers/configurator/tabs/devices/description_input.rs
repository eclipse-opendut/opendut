use leptos::prelude::*;
use opendut_lea_components::{UserInputValue, UserTextarea};
use opendut_model::topology::DeviceDescription;
use crate::peers::configurator::types::devices::UserDeviceConfiguration;

#[component]
pub fn DeviceDescriptionInput(
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
