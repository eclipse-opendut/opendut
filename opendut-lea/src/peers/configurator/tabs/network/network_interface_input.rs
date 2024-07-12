use std::ops::Not;
use leptos::*;

use opendut_types::util::net::{CanSamplePoint, NetworkInterfaceConfiguration, NetworkInterfaceName, NetworkInterfaceNameError};

use crate::components::{ButtonColor, ButtonSize, ButtonState, FontAwesomeIcon, IconButton, UserInput, UserInputValue};
use crate::peers::configurator::types::{UserNetworkInterface};
use crate::util::net::UserNetworkInterfaceConfiguration;
use crate::util::NON_BREAKING_SPACE;

#[component]
pub fn NetworkInterfaceInput<A>(
    interfaces: Signal<Vec<RwSignal<UserNetworkInterface>>>,
    on_action: A
) -> impl IntoView
where A: Fn(NetworkInterfaceName, UserNetworkInterfaceConfiguration) + 'static {

    let (getter, setter) = create_signal(UserInputValue::Left(String::from(NON_BREAKING_SPACE)));

    let (getter_type, setter_type) = create_signal("Ethernet");

    let name_filter = move |name: NetworkInterfaceName| {
        interfaces.with(|interfaces| {
            interfaces.iter()
                .cloned()
                .filter(|interface| {
                    interface.get().name == name
                })
                .collect::<Vec<_>>()
        })
    };

    let validator = move |input: String| {
        match NetworkInterfaceName::try_from(input.clone()) {
            Ok(name) => {
                if name_filter(name.clone()).is_empty().not() {
                    UserInputValue::Both("A network interface with that name has already been configured.".to_string(), input)
                } else {
                    UserInputValue::Right(input)
                }
            }
            Err(cause) => {
                match cause {
                    NetworkInterfaceNameError::TooLong { value, max } => {
                        UserInputValue::Both(format!("A network interface name must be at most {} characters long.", max), value)
                    },
                    NetworkInterfaceNameError::Empty => {
                        UserInputValue::Left(String::from(NON_BREAKING_SPACE))
                    },
                }
            }
        }
    };

    let button_state = MaybeSignal::derive(move || {
        if getter.get().is_left() || getter.get().is_both() {
            ButtonState::Disabled
        } else {
            ButtonState::Enabled
        }
    });

    view! {
        <div class="is-flex is-align-items-center">
            <div class="is-flex is-align-items-stretch">
                <UserInput
                    getter = getter.into()
                    setter = setter.into()
                    validator
                    label = "Name"
                    placeholder = "eth0"
                />
                <div class="field is-align-items-top ml-4">
                    <label class="label">Type</label>
                    <div class="control mt-4">
                        <label class="radio">
                            <input 
                                type="radio"
                                name="interfaceType"
                                checked = move || {
                                    matches!(getter_type.get(), "Ethernet")
                                }
                                on:click = move |_| {
                                    setter_type.set("Ethernet");
                                }/>
                            " Ethernet "
                        </label>
                        <label class="radio">
                            <input 
                                type="radio" 
                                name="interfaceType"
                                checked = move || {
                                    matches!(getter_type.get(), "CAN")
                                }
                                on:click = move |_| {
                                    setter_type.set("CAN");
                                }
                            />
                            " CAN "
                        </label>
                    </div>
                </div>
            </div>
            <div class="ml-4">
                <IconButton
                    icon = FontAwesomeIcon::Plus
                    color = ButtonColor::Success
                    size = ButtonSize::Normal
                    state = button_state
                    label = "Add network interface"
                    show_label = true
                    on_action = move || {
                        if let UserInputValue::Right(value) = getter.get_untracked() {
                            if let Ok(name) = NetworkInterfaceName::try_from(value) {
                                let configuration = match getter_type.get() {
                                    "Ethernet" => {
                                        NetworkInterfaceConfiguration::Ethernet
                                    }
                                    _ => {
                                        NetworkInterfaceConfiguration::Can {
                                            bitrate: 500000,
                                            sample_point: CanSamplePoint::try_from(0.7).unwrap(),
                                            fd: true,
                                            data_bitrate: 2000000,
                                            data_sample_point: CanSamplePoint::try_from(0.7).unwrap(),
                                        }
                                    }
                                };
                                let configuration = UserNetworkInterfaceConfiguration::from(configuration);
                                on_action(name, configuration);
                                setter.set(UserInputValue::Right(String::new()));
                            }
                        }
                    }
                />
            </div>
        </div>
         <td class="is-narrow" style="text-align: center">
        </td>
    }
}
