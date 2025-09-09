use std::ops::Not;
use leptos::either::Either;
use leptos::prelude::*;
use opendut_model::util::net::{CanSamplePoint, NetworkInterfaceConfiguration, NetworkInterfaceName, NetworkInterfaceNameError};

use crate::components::{ButtonColor, ButtonSize, ButtonState, FontAwesomeIcon, IconButton, UserInput, UserInputValue};
use crate::peers::configurator::types::UserNetworkInterface;
use crate::components::UserNetworkInterfaceConfiguration;
use crate::components::NON_BREAKING_SPACE;

#[component]
pub fn NetworkInterfaceInput<A>(
    interfaces: Signal<Vec<RwSignal<UserNetworkInterface>>>,
    on_action: A
) -> impl IntoView
where A: Fn(NetworkInterfaceName, UserNetworkInterfaceConfiguration) + 'static {

    let (interface_name_getter, interface_name_setter) = signal(UserInputValue::Left(String::from(NON_BREAKING_SPACE)));
    let (bitrate_getter, bitrate_setter) = signal(UserInputValue::Right(String::from("500")));
    let (sample_point_getter, sample_point_setter) = signal(UserInputValue::Right(String::from("0.7")));
    let (data_bitrate_getter, data_bitrate_setter) = signal(UserInputValue::Right(String::from("2000")));
    let (data_sample_point_getter, data_sample_point_setter) = signal(UserInputValue::Right(String::from("0.7")));

    let (getter_type, setter_type) = signal(InterfaceKind::Ethernet);
    let (can_fd_getter_type, can_fd_setter_type) = signal(false);

    let name_filter = move |name: NetworkInterfaceName| {
        interfaces.with(|interfaces| {
            interfaces.iter()
                .cloned()
                .filter(|interface| {
                    interface.get_untracked().name == name
                })
                .collect::<Vec<_>>()
        })
    };

    let name_validator = move |input: String| {
        match NetworkInterfaceName::try_from(input.trim()) {
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
                        UserInputValue::Both(format!("A network interface name must be at most {max} characters long."), value)
                    },
                    NetworkInterfaceNameError::Empty => {
                        UserInputValue::Left(String::from(NON_BREAKING_SPACE))
                    },
                }
            }
        }
    };

    let sample_points_validator = move |input: String| { sample_points_validator(input) };

    let bitrate_validator = move |input| { bitrate_validator(input) };

    let button_state = Signal::derive(move || {
        match getter_type.get() {
            InterfaceKind::Ethernet => {
                if interface_name_getter.get().is_left() || interface_name_getter.get().is_both() {
                    ButtonState::Disabled
                } else {
                    ButtonState::Enabled
                }
            }
            InterfaceKind::Can => {
                if can_fd_getter_type.get() {
                    if interface_name_getter.get().is_right()
                        && bitrate_getter.get().is_right()
                        && sample_point_getter.get().is_right()
                        && data_bitrate_getter.get().is_right()
                        && data_sample_point_getter.get().is_right() {
                        ButtonState::Enabled
                    } else {
                        ButtonState::Disabled
                    }
                } else if interface_name_getter.get().is_right()
                        && bitrate_getter.get().is_right()
                        && sample_point_getter.get().is_right() {
                    ButtonState::Enabled
                } else {
                    ButtonState::Disabled
                }
            }
        }
    });

    
    let can_fd_view = {
        move || if getter_type.get() == InterfaceKind::Can {
            Either::Right(view! {
                <div class="is-flex is-align-items-center mb-3">
                    <div class="mr-3">
                        <UserInput
                            getter = bitrate_getter.into()
                            setter = bitrate_setter.into()
                            validator = bitrate_validator
                            label = "Bitrate (kb/s)"
                            placeholder = "500"
                        />
                    </div>
                    <div>
                        <UserInput
                            getter = sample_point_getter.into()
                            setter = sample_point_setter.into()
                            validator = sample_points_validator
                            label = "Sample Point"
                            placeholder = "0.000 .. 0.999"
                        />
                    </div>
                    <label class="checkbox ml-3 mr-3">
                    <input
                        type="checkbox" 
                        name="canType"
                        checked = move || {
                            can_fd_getter_type.get()
                        }
                        on:click = move |_| {
                            can_fd_setter_type.set(!can_fd_getter_type.get());
                        }
                    />
                        " CAN FD "
                    </label>
                    {
                        move || if can_fd_getter_type.get() {
                            Either::Right(view!{
                                <div class="is-flex">
                                    <div class="mr-3">
                                        <UserInput
                                            getter = data_bitrate_getter.into()
                                            setter = data_bitrate_setter.into()
                                            validator = bitrate_validator
                                            label = "Data Bitrate (kb/s)"
                                            placeholder = "1000"
                                        />
                                    </div>
                                    <div>
                                        <UserInput
                                            getter = data_sample_point_getter.into()
                                            setter = data_sample_point_setter.into()
                                            validator = sample_points_validator
                                            label = "Data Sample Point"
                                            placeholder = "0.000 .. 0.999"
                                        />
                                    </div>
                                </div>
                            })
                        } else {
                            Either::Left(view!{ <div></div> })
                        }
                    }
                </div>
            })
        } else {
            Either::Left(view! { <div></div> })
        }
    };
    
    view! {
        <div class="is-flex is-align-items-center">
            <div class="is-flex is-align-items-stretch">
                <UserInput
                    getter = interface_name_getter.into()
                    setter = interface_name_setter.into()
                    validator = name_validator
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
                                    matches!(getter_type.get(), InterfaceKind::Ethernet)
                                }
                                on:click = move |_| {
                                    setter_type.set(InterfaceKind::Ethernet);
                                }/>
                            " Ethernet "
                        </label>
                        <label class="radio">
                            <input 
                                type="radio" 
                                name="interfaceType"
                                checked = move || {
                                    matches!(getter_type.get(), InterfaceKind::Can)
                                }
                                on:click = move |_| {
                                    setter_type.set(InterfaceKind::Can)
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
                        if let UserInputValue::Right(value) = interface_name_getter.get_untracked() {
                            if let Ok(name) = NetworkInterfaceName::try_from(value) {
                                let configuration = match getter_type.get() {
                                    InterfaceKind::Ethernet => {
                                        NetworkInterfaceConfiguration::Ethernet
                                    }
                                    InterfaceKind::Can => {
                                        let sample_point = sample_point_getter.get().right().unwrap();
                                        let data_sample_point = data_sample_point_getter.get().right().unwrap();
                                        let bitrate = bitrate_getter.get().right().unwrap();
                                        let data_bitrate = data_bitrate_getter.get().right().unwrap();

                                        NetworkInterfaceConfiguration::Can {
                                            bitrate: bitrate.parse::<u32>().unwrap() * 1000,
                                            sample_point: CanSamplePoint::try_from(sample_point.parse::<f32>().unwrap()).unwrap(),
                                            fd: can_fd_getter_type.get(),
                                            data_bitrate: data_bitrate.parse::<u32>().unwrap() * 1000,
                                            data_sample_point: CanSamplePoint::try_from(data_sample_point.parse::<f32>().unwrap()).unwrap(),
                                        }
                                    }
                                };
                                let configuration = UserNetworkInterfaceConfiguration::from(configuration);
                                on_action(name, configuration);
                                interface_name_setter.set(UserInputValue::Right(String::new()));
                            }
                        }
                    }
                />
            </div>
        </div>
        {
            can_fd_view
        }
        <td class="is-narrow" style="text-align: center">
        </td>
    }
}

fn bitrate_validator(input: String) -> UserInputValue {
    match input.parse::<u32>() {
        Ok(_bitrate) => {
            UserInputValue::Right(input)
        }
        Err(_cause) => {
            UserInputValue::Both("Could not parse String into u32.".to_string(), input)
        }
    }
}

fn sample_points_validator(input: String) -> UserInputValue {
    let sample_point_parsed = input.parse::<f32>();
    match sample_point_parsed {
        Ok(sample_point) => {
            if (0.0..1.0).contains(&sample_point) {
                match CanSamplePoint::try_from(sample_point) {
                    Ok(_can_sample_point) => {
                        UserInputValue::Right(input)
                    }
                    Err(_cause) => {
                        UserInputValue::Both("Not a valid sample point.".to_string(), input)
                    }
                }
            } else {
                UserInputValue::Both("Range must be between 0.000 and 0.999.".to_string(), input)
            }
        }
        Err(_cause) => {
            UserInputValue::Both("Range must be between 0.000 and 0.999.".to_string(), input)
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
enum InterfaceKind {
    Ethernet,
    Can,
}


#[cfg(test)]
mod test {
    use crate::peers::configurator::tabs::network::network_interface_input::{bitrate_validator, sample_points_validator};

    #[test]
    fn test_bitrate_validator_succeeds() {
        let input = "500".to_string();
        let validator_function = bitrate_validator(input);
        assert!(validator_function.is_right());
    }

    #[test]
    fn test_bitrate_validator_fails() {
        let input = "5000abc".to_string();
        let validator_function = bitrate_validator(input);
        assert!(validator_function.is_both());

        let input = "".to_string();
        let validator_function = bitrate_validator(input);
        assert!(validator_function.is_both());

        let input = " ".to_string();
        let validator_function = bitrate_validator(input);
        assert!(validator_function.is_both());
    }

    #[test]
    fn test_sample_points_validator_succeeds() {
        let input = "0.000".to_string();
        let validator_function = sample_points_validator(input);
        assert!(validator_function.is_right());
    }

    #[test]
    fn test_sample_points_validator_fails() {
        let input = "1.0".to_string();
        let validator_function = sample_points_validator(input);
        assert!(validator_function.is_both());

        let input = "-5.05".to_string();
        let validator_function = sample_points_validator(input);
        assert!(validator_function.is_both());

        let input = "".to_string();
        let validator_function = sample_points_validator(input);
        assert!(validator_function.is_both());

        let input = " ".to_string();
        let validator_function = sample_points_validator(input);
        assert!(validator_function.is_both());
    }
}
