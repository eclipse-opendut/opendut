use leptos::prelude::*;
use opendut_lea_components::{SelectionTable, SelectionTableRow};
use opendut_model::viper::ViperSourceDescriptor;
use crate::app::use_app_globals;
use crate::viper_tests::configurator::types::{SourceSelection, UserViperTestConfiguration};

#[component]
pub fn ViperTestSourceSelector(viper_test_configuration: RwSignal<UserViperTestConfiguration>) -> impl IntoView {

    let globals = use_app_globals();

    let viper_sources = {
        let carl = globals.client.clone();

        LocalResource::new(move || {
            let mut carl = carl.clone();
            async move {
                carl.viper.list_viper_source_descriptors().await
                    .expect("Failed to request the list of viper_sources")
            }
        })
    };

    let (getter, setter) = create_slice(viper_test_configuration,
        |config| {
            Clone::clone(&config.viper_source)
        },
        |config, input| {
            config.viper_source = input;
        }
    );

    let viper_sources = Signal::derive(move || {
        if let Some(mut viper_sources) = viper_sources.get() {
            viper_sources.sort_by(|viper_source_a, viper_source_b| {
                viper_source_a.name.value().to_lowercase()
                    .cmp(&viper_source_b.name.value().to_lowercase())
            });

            let rows = viper_sources.iter().map(|viper_source_descriptor| {
                let ViperSourceDescriptor { id, name, url } = viper_source_descriptor;
                let id = id.to_owned();
                let name = name.to_string();
                let url = url.to_string();

                SelectionTableRow {
                    id,
                    cells: vec![name, url]
                }
            }).collect::<Vec<_>>();

            if viper_sources.is_empty() {
                setter.set(SourceSelection::Left(String::from("No viper viper_sources available.")));
            } else if matches!(getter.get(), SourceSelection::Left(_)) {
                setter.set(SourceSelection::Left(String::from("Select a viper source.")));
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
            rows=viper_sources
            getter
            setter
        />
    }
}
