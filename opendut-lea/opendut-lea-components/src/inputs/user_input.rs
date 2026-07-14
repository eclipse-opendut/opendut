use leptos::prelude::*;
use leptos::reactive::wrappers::write::SignalSetter;
use web_sys::KeyboardEvent;
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
    #[prop(optional)] on_enter: Option<Callback<()>>,
    #[prop(into, default=Signal::from(String::from(NON_BREAKING_SPACE)))] empty_help_text: Signal<String>,
) -> impl IntoView
where A: UserInputValidator + Clone + Send + Sync + 'static {

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
            UserInputValue::Right(_) => empty_help_text.get(),
            UserInputValue::Left(error) => error.to_owned(),
            UserInputValue::Both(error, _) => error.to_owned(),
        })
    };

    let aria_label = Clone::clone(&label);

    let validate_value = Callback::new(move |value| {
        if let Some(validator) = &validator {
            let validated_value = validator.validate(value);
            setter.set(validated_value);
        }
        else {
            setter.set(UserInputValue::Right(value));
        }
    });

    let debounced_input_handling = leptos_use::use_debounce_fn_with_arg(
        move |ev| {
            validate_value.run(event_target_value(&ev));
        },
        INPUT_VALIDATION_DEBOUNCE_MS,
    );

    let run_on_enter = move |ev: KeyboardEvent| {
        if ev.key() == "Enter" {
            ev.prevent_default();

            let input = event_target_value(&ev);
            validate_value.run(input);

            if let Some(on_enter) = &on_enter {
                on_enter.run(());
            }
        }
    };

    view! {
        <div>
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
                        on:keydown=move |ev| { run_on_enter(ev); }
                    />
                </div>
                { add_on.map(|add_on_button| add_on_button.run()) }
            </div>
            <p class="help has-text-danger mb-3">{ help_text }</p>
        </div>
    }
}
