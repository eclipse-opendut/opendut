use std::ops::Not;
use leptos::*;

use opendut_types::util::net::{NetworkInterfaceName, NetworkInterfaceNameError};

use crate::components::{ButtonColor, ButtonSize, ButtonState, FontAwesomeIcon, IconButton, UserInput, UserInputValue};
use crate::peers::configurator::types::UserPeerNetworkInterface;
use crate::util::NON_BREAKING_SPACE;

#[component]
pub fn NetworkInterfaceNameInput<A>(
    interfaces: Signal<Vec<RwSignal<UserPeerNetworkInterface>>>,
    on_action: A
) -> impl IntoView
where A: Fn(NetworkInterfaceName) + 'static {

    let (getter, setter) = create_signal(UserInputValue::Left(String::from(NON_BREAKING_SPACE)));

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
        match NetworkInterfaceName::try_from(input) {
            Ok(name) => {
                if name_filter(name.clone()).is_empty().not() {
                    UserInputValue::Both("A network interface with that name has already been configured.".to_string(), name.name())
                } else {
                    UserInputValue::Right(name.name())
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
            <UserInput
                getter = getter.into()
                setter = setter.into()
                validator
                label = "Network Interface Name"
                placeholder = "eth0"
            />
            <div class="ml-2">
                <IconButton
                    icon = FontAwesomeIcon::Plus
                    color = ButtonColor::Success
                    size = ButtonSize::Normal
                    state = button_state
                    label = "Add network interface"
                    on_action = move || {
                        if let UserInputValue::Right(value) = getter.get_untracked() {
                            if let Ok(name) = NetworkInterfaceName::try_from(value) {
                                on_action(name);
                                setter.set(UserInputValue::Right(String::new()));
                            }
                        }
                    }
                />
            </div>
        </div>
    }
}
