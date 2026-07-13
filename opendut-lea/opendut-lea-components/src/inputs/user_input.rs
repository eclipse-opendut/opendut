use leptos::prelude::*;
use leptos::reactive::wrappers::write::SignalSetter;
use crate::inputs::{InputType, UserInputValidator, UserInputValue};

use crate::NON_BREAKING_SPACE;

const INPUT_VALIDATION_DEBOUNCE_MS: f64 = 300.0;

#[component]
pub fn UserInput<A>(
    getter: Signal<UserInputValue>,
    setter: SignalSetter<UserInputValue>,
    #[prop(optional)] validator: Option<A>,
    #[prop(into)] label: Signal<String>,
    #[prop(into)] placeholder: Signal<String>,
    #[prop(into, default=Signal::from(None))] description: Signal<Option<String>>,
    #[prop(default=InputType::Text)] input_type: InputType,
    #[prop(optional)] add_on: Option<ViewFn>,
) -> impl IntoView
where A: UserInputValidator + Clone + 'static {

    let has_description = move || description.with(|description| description.is_some());
    let has_add_on = {
        let add_on = add_on.clone();
        move || add_on.is_some()
    };

    let value_text = move || {
        getter.with(|input| match input {
            UserInputValue::Left(_) => String::new(),
            UserInputValue::Right(value) => value.to_owned(),
            UserInputValue::Both(_, value) => value.to_owned(),
        })
    };

    let help_text = move || {
        getter.with(|input| match input {
            UserInputValue::Right(_) => String::from(NON_BREAKING_SPACE),
            UserInputValue::Left(error) => error.to_owned(),
            UserInputValue::Both(error, _) => error.to_owned(),
        })
    };

    let aria_label = Clone::clone(&label);

    let debounced_input_handling = leptos_use::use_debounce_fn_with_arg(
        move |ev| {
            if let Some(validator) = &validator {
                let validated_value = validator.validate(event_target_value(&ev));
                setter.set(validated_value);
            }
            else {
                let target_value = event_target_value(&ev);
                setter.set(UserInputValue::Right(target_value));
            }
        },
        INPUT_VALIDATION_DEBOUNCE_MS,
    );

    view! {
        <label class="label" class=("mb-0", has_description)>{ label }</label>
        <div class="field mb-0" class=("has-addons", has_add_on)>
            <Show when=has_description>
                <p class="pb-2"> { description } </p>
            </Show>
            <div class="control">
                <input
                    class="input"
                    type=input_type.as_html_type()
                    aria-label=move || aria_label.get()
                    placeholder=move || placeholder.get()
                    prop:value={ value_text }
                    on:input=move |ev| { debounced_input_handling(ev); }
                />
            </div>
            { add_on.map(|add_on_button| add_on_button.run()) }
        </div>
        <p class="help has-text-danger mb-3">{ help_text }</p>
    }
}
