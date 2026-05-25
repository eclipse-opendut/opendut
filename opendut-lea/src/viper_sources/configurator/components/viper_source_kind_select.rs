use leptos::{either::Either, prelude::*};
use opendut_model::viper::ViperSourceKind;
use crate::viper_sources::configurator::types::UserViperSourceConfiguration;

#[component]
pub fn ViperSourceKindSelect(viper_source_configuration: RwSignal<UserViperSourceConfiguration>) -> impl IntoView {

    let (getter, setter) = create_slice(viper_source_configuration,
        |config| {
            Clone::clone(&config.kind)
        },
        |config, value| {
            config.kind = value;
        }
    );

    let value = move || match getter.get() {
        ViperSourceKind::Git => "Git",
        ViperSourceKind::Http => "Http",
    };

    let dropdown_options = move || {
        let options = [("Git", ViperSourceKind::Git), ("Http", ViperSourceKind::Http)];
        options.into_iter()
            .map(|(label, _)| {
                if label == value() {
                    Either::Left(view! {
                        <option selected>{label}</option>
                    })
                } else {
                    Either::Right(view! {
                        <option>{label}</option>
                    })
                }
            })
            .collect::<Vec<_>>()
    };

    view! {
        <div class="field pb-3">
            <label class="label">"Source Type"</label>
            <div class="control">
                <div class="select"
                    on:change=move |ev| {
                        let target_value = event_target_value(&ev);
                        match target_value.as_str() {
                            "Git" => { setter.set(ViperSourceKind::Git); }
                            "Http" => { setter.set(ViperSourceKind::Http); }
                            _ => {}
                        };
                    }>
                    <select>
                        { dropdown_options }
                    </select>
                </div>
            </div>
        </div>
    }
}
