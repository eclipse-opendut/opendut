use leptos::prelude::*;
use opendut_lea_components::{Toggle, ToggleSignal, ToggleState};
use opendut_model::viper::ViperBindingValue;
use crate::viper_tests::configurator::types::ViperBindingValueInput;

#[component]
pub fn BooleanParameterInput(
    getter: Signal<Option<ViperBindingValueInput>>,
    setter: SignalSetter<ViperBindingValueInput>,
    name: String,
    display_name: Option<String>,
    description: Option<String>,
    use_default_value: Option<RwSignal<bool>>,
    default: bool,
) -> impl IntoView {

    let is_active = Signal::derive(move || {
        match getter.get() {
            Some(ViperBindingValueInput::Right(Some(ViperBindingValue::BooleanValue(value)))) => value,
            _ => default,
        }
    });

    let on_toggle = move || {
        let is_active = is_active.get();
        let value = ViperBindingValueInput::Right(
            Some(ViperBindingValue::BooleanValue(!is_active))
        );

        setter.set(value);
    };
    
    let toggle_state = Signal::derive(move || {
        if use_default_value.get().unwrap_or(false) {
            ToggleState::Disabled
        } else {
            ToggleState::Enabled
        }
    });

    view! {
        <div class="field mb-5">
            <div class="is-flex control">
                <Toggle
                    right_text=display_name.unwrap_or_else(|| name)
                    has_bold_text=true
                    state=toggle_state
                    is_active
                    on_action=on_toggle
                />
            </div>
            { description }
            {
                if let Some(use_default_value) = use_default_value {
                    Some(view! {
                        <div class="is-flex is-justify-content-start">
                            <Toggle
                                left_text="Use default:"
                                is_active=use_default_value
                                on_action=move || {
                                    use_default_value.toggle();
                                }
                            />
                        </div>
                    })
                } else { None }
            }
        </div>
    }
}
