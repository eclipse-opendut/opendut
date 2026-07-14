use leptos::prelude::*;
use opendut_lea_components::{use_toaster, ButtonColor, ButtonState, SimpleButton, UserInput, UserInputValue, Toast, Tag, NON_BREAKING_SPACE};
use opendut_model::topology::{DeviceTag, IllegalDeviceTag};
use crate::peers::configurator::types::devices::UserDeviceConfiguration;

#[component]
pub fn DeviceTagInput(
    device_configuration: RwSignal<UserDeviceConfiguration>,
) -> impl IntoView {
    let toaster = use_toaster();

    let (tags_getter, tags_setter) = create_slice(device_configuration,
        |device_configuration| {
            Clone::clone(&device_configuration.tags)
        },
        |device_configuration, value| {
            device_configuration.tags = value;
        }
    );

    let (input_getter, input_setter) = signal(UserInputValue::Right(String::new()));

    let validator = move |input: String| {
        let tags = tags_getter.get();
        let already_exists = tags.iter()
            .any(|tag| tag.value() == input);

        if already_exists {
            return UserInputValue::Both(String::from("This tag already exists."), input);
        }

        match DeviceTag::try_from(input.clone()) {
            Ok(_) => {
                UserInputValue::Right(input)
            }
            Err(cause) => {
                match cause {
                    IllegalDeviceTag::TooLong { expected, value, .. } => {
                        UserInputValue::Both(format!("A tag must be at most {expected} characters long."), value)
                    }
                }
            }
        }
    };

    let add_tag = Callback::new(move |_| {
        let input = input_getter.get();
        let toaster = toaster.clone();

        if let UserInputValue::Right(input) = input && !input.trim().is_empty() {
            let mut tags = tags_getter.get();
            let input_tag = DeviceTag::try_from(input.clone());

            match input_tag {
                Ok(input_tag) => {
                    tags.push(input_tag);
                    tags_setter.set(tags);
                    input_setter.set(UserInputValue::Right(String::new()));
                }
                Err(error) => {
                    toaster.toast(
                        Toast::builder()
                            .simple(format!("Failed to add tag, due to error: {}", error))
                            .error()
                    )
                }
            }
        }
    });

    view! {
        <UserInput
            getter=input_getter.into()
            setter=input_setter.into()
            validator
            label="Tags"
            placeholder="automotive"
            empty_help_text=Signal::derive(move || if tags_getter.get().is_empty() { String::from(NON_BREAKING_SPACE) } else { String::new() })
            on_enter=add_tag
            add_on = ViewFn::from(move || {
                view! {
                    <AddOnButton
                        input_getter
                        add_tag
                    />
                }.into_any()
            })
        />

        <div class="field is-grouped is-grouped-multiline">
            <For
                each=move || tags_getter.get()
                key=|tag| tag.value().to_owned()
                children=move |device_tag| {
                    view! {
                        <Tag
                            text=device_tag.value()
                            on_delete=Callback::new(move |_| {
                                let mut tags = tags_getter.get();
                                tags.retain(|tag| tag.value() != device_tag.value());
                                tags_setter.set(tags);
                            })
                        />
                    }
                }
            />
        </div>
    }
}

#[component]
fn AddOnButton(
    input_getter: ReadSignal<UserInputValue>,
    add_tag: Callback<()>,
) -> impl IntoView {

    let button_state = move || {
        if let UserInputValue::Right(input) = input_getter.get()
            && !input.trim().is_empty() {
            ButtonState::Enabled
        }
        else {
            ButtonState::Disabled
        }
    };

    view! {
        <div class="control">
            <SimpleButton
                text="Add"
                color=ButtonColor::Info
                state=button_state
                on_action=move || add_tag.run(())
            />
        </div>
    }
}
