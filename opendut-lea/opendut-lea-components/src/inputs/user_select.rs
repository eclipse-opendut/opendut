use leptos::prelude::*;
use crate::UserInputValue;

#[derive(Clone, Debug)]
pub struct SelectionOption {
    pub display_name: String,
    pub value: String,
}

#[component]
pub fn UserSelect(
    #[prop(into)] options: Signal<Vec<SelectionOption>>,
    #[prop(into)] initial_option: Signal<String>,
    getter: Signal<UserInputValue>,
    setter: SignalSetter<UserInputValue>,
    #[prop(into)] label: Signal<String>,
) -> impl IntoView {

    let selected_value = move || {
        getter.with(|input| match input {
            UserInputValue::Left(_) => initial_option.get(),
            UserInputValue::Right(value) => value.to_owned(),
            UserInputValue::Both(_, value) => value.to_owned(),
        })
    };

    // let help_text = move || {
    //     getter.with(|input| match input {
    //         UserInputValue::Right(_) => String::from(NON_BREAKING_SPACE),
    //         UserInputValue::Left(error) => error.to_owned(),
    //         UserInputValue::Both(error, _) => error.to_owned(),
    //     })
    // };

    let has_error = move || {
        getter.with(|input| {
            matches!(input, UserInputValue::Left(_))
        })
    };

    view! {
        <div class="field">
            <label class="label">{ label }</label>
            <div class="control">
                <div class="select" class:is-danger=has_error>
                    <select
                        aria-label=move || label.get()
                        prop:value=selected_value
                        on:change=move |ev| {
                            let target_value = event_target_value(&ev);
                            if target_value == initial_option.get() {
                                setter.set(UserInputValue::Left(initial_option.get()));
                            } else {
                                setter.set(UserInputValue::Right(target_value));
                            }
                        }
                    >
                    <option> { initial_option } </option>
                        <For
                            each=move || options.get()
                            key=|option| option.value.to_owned()
                            children=move |option| {
                                let option_value = option.value;
                                let display_name = option.display_name;
                                view! {
                                    <option value=option_value>
                                        { display_name }
                                    </option>
                                }
                            }
                        />
                    </select>
                </div>
                // <p class="help has-text-danger">{ help_text }</p>
            </div>
        </div>
    }
}
