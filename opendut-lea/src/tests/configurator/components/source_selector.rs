use leptos::prelude::*;
use opendut_lea_components::{UserInputValue, NON_BREAKING_SPACE};
use opendut_model::viper::ViperSourceId;
use crate::app::use_app_globals;
use crate::tests::configurator::types::UserTestConfiguration;

#[component]
pub fn TestSourceSelector(test_configuration: RwSignal<UserTestConfiguration>) -> impl IntoView {

    let globals = use_app_globals();

    let registered_sources = {
        let carl = globals.client.clone();

        LocalResource::new(move || {
            let mut carl = carl.clone();
            async move {
                carl.viper.list_viper_source_descriptors().await
                    .expect("Failed to request the list of sources")
            }
        })
    };

    let (getter, setter) = create_slice(test_configuration,
        |config| {
            Clone::clone(&config.source)
        },
        |config, input| {
            config.source = input;
        }
    );

    let help_text = move || {
        getter.with(|selection| match selection {
            UserInputValue::Left(error) => error.to_owned(),
            UserInputValue::Right(_) => String::from(NON_BREAKING_SPACE),
            UserInputValue::Both(error, _) => error.to_owned(),
        })
    };

    let sources = Signal::derive(move || {
        if let Some(mut sources) = registered_sources.get() {
            sources.sort_by(|source_a, source_b| {
                source_a.name.value().to_lowercase()
                    .cmp(&source_b.name.value().to_lowercase())
            });

            if sources.is_empty() {
                setter.set(UserInputValue::Left(String::from("No sources available.")));
            } else if matches!(getter.get(), UserInputValue::Left(_)) {
                setter.set(UserInputValue::Left(String::from("Select a source.")));
            }

            sources
        } else {
            Vec::new()
        }
    });

    let is_selected = move |source: ViperSourceId| {
        let source = Clone::clone(&source);
        Signal::derive(move || {
            let getter = getter.get();
            match getter {
                UserInputValue::Right(selected) => source.to_string() == selected,
                UserInputValue::Left(_) | UserInputValue::Both(_, _) => false,
            }
        })
    };

    view! {
        <p class="help has-text-danger"> { help_text } </p>
        <div class="table-container mt-2">
            <table class="table is-fullwidth">
                <thead>
                    <tr>
                        <th>Name</th>
                        <th>URL</th>
                        <th></th>
                    </tr>
                </thead>
                <tbody>
                    <For
                        each=move || sources.get()
                        key=|source| source.id
                        children=move |source| {
                            let source_id = source.id;
                            let source_href = move || format!("/sources/{}/configure/general", source_id);

                            let is_selected = is_selected(Clone::clone(&source_id));

                            view! {
                                <tr
                                    class:has-background-link-light=move || is_selected.get()
                                    style="cursor: pointer;"
                                    on:click=move |_| {
                                        setter.set(UserInputValue::Right(source_id.to_string()));
                                    }
                                >
                                    <td>
                                        <a href=source_href>
                                            { source.name.to_string() }
                                        </a>
                                    </td>
                                    <td>
                                        { source.url.to_string() }
                                    </td>
                                    <td class="is-narrow" style="text-align: center">
                                        <div class="control">
                                            <label class="radio">
                                                <input
                                                    type="radio"
                                                    name="selected-source"
                                                    prop:checked=is_selected
                                                    on:click=move |_| {
                                                        setter.set(UserInputValue::Right(source_id.to_string()));
                                                    }
                                                />
                                            </label>
                                        </div>
                                    </td>
                                </tr>
                            }
                        }
                    />
                </tbody>
            </table>
        </div>
    }
}
