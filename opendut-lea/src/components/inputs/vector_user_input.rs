use leptos::*;
use crate::components::{ButtonColor, ButtonSize, ButtonState, ConfirmationButton, FontAwesomeIcon, UserInputError};
use crate::components::inputs::{UserInputValidator, UserInputValue};
use crate::util::{Ior, NON_BREAKING_SPACE};

#[component]
pub fn VectorUserInput<OnAddFn>(
    getter: Signal<Vec<RwSignal<UserInputValue>>>,
    setter: SignalSetter<Vec<RwSignal<UserInputValue>>>,
    validator: fn(String) -> Ior<UserInputError, String>,
    #[prop(into)] label: MaybeSignal<String>,
    #[prop(into)] placeholder: MaybeSignal<String>,
    #[prop(into)] delete_label: MaybeSignal<String>,
    on_add: OnAddFn
) -> impl IntoView
where 
    OnAddFn: Fn() + 'static
{
    let on_value_delete = move |delete_value: String| {
        let remaining_values= getter.with_untracked(|values| {
            values.iter()
                .filter(|values| values.with_untracked(|value| {
                    let value = match value {
                        UserInputValue::Left(_) => String::new(),
                        UserInputValue::Right(value) => value.to_owned(),
                        UserInputValue::Both(_, value) => value.to_owned(),
                    };
                    value != delete_value
                }))
                .cloned()
                .collect::<Vec<_>>()
        });
        setter.set(remaining_values)
    };

    let aria_label = Clone::clone(&label);

    let panels = create_memo(move |_| {
        getter.with(|inputs| {
            inputs.iter()
                .cloned()
                .map(|input| {
                    let (panel_getter, panel_setter) = create_slice(input,
                            |input| {
                                Clone::clone(input)
                            },
                            |input, value| {
                                *input = value;
                            }
                    );

                    view! {
                        <VectorUserInputValue
                            getter=panel_getter
                            setter=panel_setter
                            label=aria_label.get()
                            placeholder=placeholder.get()
                            validator
                            delete_label=delete_label.get()
                            on_delete=on_value_delete
                        />
                    }
                })
                .collect::<Vec<_>>()
        })
    });

    view! {
        <div>
             <div class="field">
                <label class="label">{ label }</label>
                { panels }
            </div>
            <div>
                <div
                    class="dut-panel-ghost has-text-success px-4 py-3 is-clickable is-flex is-justify-content-center mb-5"
                    on:click=move |_| {
                       on_add()
                    }
                >
                    <span><i class="fa-solid fa-circle-plus"></i></span>
                </div>
            </div>
        </div>
    }

}

#[component]
fn VectorUserInputValue<A,OnDeleteFn>(
    getter: Signal<UserInputValue>,
    setter: SignalSetter<UserInputValue>,
    #[prop(optional)] validator: Option<A>,
    #[prop(into)] label: MaybeSignal<String>,
    #[prop(into)] placeholder: MaybeSignal<String>,
    #[prop(into)] delete_label: MaybeSignal<String>,
    on_delete: OnDeleteFn
) -> impl IntoView
    where A: UserInputValidator + 'static,
          OnDeleteFn: Fn(String) + 'static
{
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

    view! {
        <div>
            <div class="control is-flex">
                <input
                    class="input"
                    type="text"
                    aria-label=move || aria_label.get()
                    placeholder=move || placeholder.get()
                    prop:value={ value_text }
                    on:input=move |ev| {
                        if let Some(validator) = &validator {
                            let validated_value = validator.validate(event_target_value(&ev));
                            setter.set(validated_value);
                        }
                        else {
                            let target_value = event_target_value(&ev);
                            setter.set(UserInputValue::Right(target_value));
                        }
                    }
                />
                <ConfirmationButton
                    icon=FontAwesomeIcon::TrashCan
                    color=ButtonColor::Light
                    size=ButtonSize::Normal
                    state=ButtonState::Enabled
                    label=delete_label.get()
                    on_conform=move || {
                        let value = match getter.get_untracked() {
                            UserInputValue::Left(_) => String::new(),
                            UserInputValue::Right(value) => value.to_owned(),
                            UserInputValue::Both(_,value) => value.to_owned(),
                        };
                        on_delete(value)
                    }
                />
            </div>
            <p class="help has-text-danger">{ help_text }</p>
        </div>
    }
}

