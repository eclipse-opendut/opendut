use leptos::{either::Either, prelude::*};
use opendut_model::secret::SecretId;
use opendut_model::viper::ViperSourceKind;
use crate::app::use_app_globals;
use crate::routing;
use crate::viper_sources::configurator::types::UserViperSourceConfiguration;

#[component]
pub fn ViperSourceSecretSelect(viper_source_configuration: RwSignal<UserViperSourceConfiguration>) -> impl IntoView {

    let globals = use_app_globals();

    let secrets_resource = {
        let carl = globals.client.clone();
        LocalResource::new(move || {
            let mut carl = carl.clone();
            async move {
                carl.secret.list_secret_descriptors().await
                    .unwrap_or_default()
            }
        })
    };

    let (kind_getter, _) = create_slice(viper_source_configuration,
        |config| Clone::clone(&config.kind),
        |config, value| { config.kind = value; }
    );

    let (secret_id_getter, secret_id_setter) = create_slice(viper_source_configuration,
        |config| Clone::clone(&config.secret_id),
        |config, value: Option<SecretId>| { config.secret_id = value; }
    );

    let is_visible = Signal::derive(move || kind_getter.get() == ViperSourceKind::Git);

    let available_secrets = Signal::derive(move || {
        secrets_resource.get().unwrap_or_default()
    });

    let dropdown_options = move || {
        let selected_id = secret_id_getter.get();
        let mut options = Vec::new();

        // "None" option
        let none_value = String::new();
        let none_label = String::from("-- No secret --");
        if selected_id.is_none() {
            options.push(Either::Left(view! {
                <option value={none_value} selected>{none_label}</option>
            }));
        } else {
            options.push(Either::Right(view! {
                <option value={none_value}>{none_label}</option>
            }));
        }

        if let Some(secrets) = secrets_resource.get() {
            for secret in secrets {
                let id_string = secret.id.to_string();
                let name = secret.name.value().to_owned();
                let is_selected = selected_id == Some(secret.id);
                if is_selected {
                    options.push(Either::Left(view! {
                        <option value={id_string} selected>{name}</option>
                    }));
                } else {
                    options.push(Either::Right(view! {
                        <option value={id_string}>{name}</option>
                    }));
                }
            }
        }

        options
    };

    view! {
        <div class="field pb-3" class:is-hidden=move || !is_visible.get()>
            <label class="label">"Secret"</label>
            <div class="control">
                {move || {
                    if available_secrets.get().is_empty() {
                        view! {
                            <div>
                                <p class="help">"No secrets available. "</p>
                                <a href=routing::path::secrets_overview class="button is-small is-link is-outlined mt-1">
                                    "Create Secret"
                                </a>
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <div class="select"
                                on:change=move |ev| {
                                    let target_value = event_target_value(&ev);
                                    if target_value.is_empty() {
                                        secret_id_setter.set(None);
                                    } else if let Ok(id) = SecretId::try_from(target_value.as_str()) {
                                        secret_id_setter.set(Some(id));
                                    }
                                }>
                                <select>
                                    { dropdown_options }
                                </select>
                            </div>
                        }.into_any()
                    }
                }}
            </div>
        </div>
    }
}
