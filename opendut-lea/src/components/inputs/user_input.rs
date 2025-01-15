use leptos::prelude::*;
use leptos::reactive::wrappers::write::SignalSetter;
use crate::components::inputs::{UserInputValidator, UserInputValue};

use crate::util::NON_BREAKING_SPACE;

const INPUT_VALIDATION_DEBOUNCE_MS: f64 = 300.0;

#[component]
pub fn UserInput<A>(
    getter: Signal<UserInputValue>,
    setter: SignalSetter<UserInputValue>,
    #[prop(optional)] validator: Option<A>,
    #[prop(into)] label: Signal<String>,
    #[prop(into)] placeholder: Signal<String>,
) -> impl IntoView
where A: UserInputValidator + Clone + 'static {

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
        <div class="field">
            <label class="label">{ label }</label>
            <div class="control">
                <input
                    class="input"
                    type="text"
                    aria-label=move || aria_label.get()
                    placeholder=move || placeholder.get()
                    prop:value={ value_text }
                    on:input=move |ev| { debounced_input_handling(ev); }
                />
            </div>
            <p class="help has-text-danger">{ help_text }</p>
        </div>
    }
}
