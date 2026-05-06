use leptos::prelude::*;
use opendut_lea_components::Toggle;
use opendut_model::viper::ViperBindingValue;
use crate::viper_tests::configurator::types::ViperBindingValueInput;

#[component]
pub fn BooleanParameterInput(
    getter: Signal<Option<ViperBindingValueInput>>,
    setter: SignalSetter<ViperBindingValueInput>,
    name: String,
    display_name: Option<String>,
    description: Option<String>,
    default: Option<bool>,
) -> impl IntoView {

    let is_active = Signal::derive(move || {
        match getter.get() {
            Some(ViperBindingValueInput::Right(ViperBindingValue::BooleanValue(value))) => value,
            _ => default.unwrap_or(false),
        }
    });

    let on_toggle = move || {
        let is_active = is_active.get();
        let value = ViperBindingValueInput::Right(
            ViperBindingValue::BooleanValue(!is_active)
        );

        setter.set(value);
    };

    view! {
        <div class="field mb-5">
            <div class="is-flex">
                <Toggle
                    text=display_name.unwrap_or_else(|| name)
                    has_bold_text=true
                    is_active
                    on_action=on_toggle
                />
            </div>
            { description }
        </div>
    }
}
