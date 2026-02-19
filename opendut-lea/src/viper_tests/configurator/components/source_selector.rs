use leptos::prelude::*;
use opendut_lea_components::{SelectionTable, SelectionTableRow};
use opendut_model::viper::ViperSourceDescriptor;
use crate::app::use_app_globals;
use crate::viper_tests::configurator::types::{SourceSelection, UserTestConfiguration};

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

    let sources = Signal::derive(move || {
        if let Some(mut sources) = registered_sources.get() {
            sources.sort_by(|source_a, source_b| {
                source_a.name.value().to_lowercase()
                    .cmp(&source_b.name.value().to_lowercase())
            });

            let rows = sources.iter().map(|source_descriptor| {
                let ViperSourceDescriptor { id, name, url } = source_descriptor;
                let id = id.to_owned();
                let name = name.to_string();
                let url = url.to_string();

                SelectionTableRow {
                    id,
                    cells: vec![name, url]
                }
            }).collect::<Vec<_>>();

            if sources.is_empty() {
                setter.set(SourceSelection::Left(String::from("No sources available.")));
            } else if matches!(getter.get(), SourceSelection::Left(_)) {
                setter.set(SourceSelection::Left(String::from("Select a source.")));
            }

            rows
        } else {
            Vec::new()
        }
    });

    let header = vec![
        String::new(),
        String::from("Name"),
        String::from("URL")
    ];

    view! {
        <SelectionTable
            header
            rows=sources
            getter
            setter
        />
    }
}
