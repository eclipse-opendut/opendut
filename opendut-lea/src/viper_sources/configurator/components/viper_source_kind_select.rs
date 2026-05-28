use leptos::prelude::*;
use opendut_model::viper::ViperSourceKind;
use crate::viper_sources::configurator::types::UserViperSourceConfiguration;

#[component]
pub fn ViperSourceKindSelect(viper_source_configuration: RwSignal<UserViperSourceConfiguration>) -> impl IntoView {

    let (getter, setter) = create_slice(viper_source_configuration,
        |config| {
            config.kind.clone()
        },
        |config, input| {
            config.kind = input;
        }
    );

    let selected_value = move || {
        match getter.get() {
            ViperSourceKind::Git => "git".to_string(),
            ViperSourceKind::Http => "http".to_string(),
        }
    };

    view! {
        <div class="field">
            <label class="label">"Source Kind"</label>
            <div class="control">
                <div class="select">
                    <select
                        aria-label="Source Kind"
                        prop:value=selected_value
                        on:change=move |ev| {
                            let target_value = event_target_value(&ev);
                            match target_value.as_str() {
                                "git" => setter.set(ViperSourceKind::Git),
                                _ => setter.set(ViperSourceKind::Http),
                            }
                        }
                    >
                        <option value="http" selected=move || matches!(getter.get(), ViperSourceKind::Http)>
                            "HTTP"
                        </option>
                        <option value="git" selected=move || matches!(getter.get(), ViperSourceKind::Git)>
                            "Git"
                        </option>
                    </select>
                </div>
            </div>
        </div>
    }
}
