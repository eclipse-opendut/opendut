use leptos::prelude::*;
use opendut_lea_components::{DefaultValue, Toggle, ToggleState};
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
    default_value: bool,
) -> impl IntoView {

    let is_active = Signal::derive(move || {
        match getter.get() {
            Some(ViperBindingValueInput::Right(Some(ViperBindingValue::BooleanValue(value)))) => value,
            _ => default_value,
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
        <DefaultValue
            default_value=default_value.to_string()
            use_default_value=use_default_value.unwrap_or(RwSignal::new(false))
        >
            <div class="field">
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
            </div>
        </DefaultValue>
    }
}
