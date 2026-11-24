use leptos::prelude::*;
use opendut_lea_components::{SelectionTable, SelectionTableRow};
use opendut_model::viper::ViperSourceDescriptor;
use crate::viper_tests::configurator::types::{SourceSelection, UserViperTestRunDescriptor};

#[component]
pub fn ViperTestSourceSelector(
    user_test_run_descriptor: RwSignal<UserViperTestRunDescriptor>,
    viper_sources: Vec<ViperSourceDescriptor>,
) -> impl IntoView {

    let (getter, setter) = create_slice(user_test_run_descriptor,
        |config| {
            Clone::clone(&config.viper_source)
        },
        |config, input| {
            config.viper_source = input;
        }
    );

    let has_sources = !viper_sources.is_empty();
    Effect::new(move || {
        let selection = getter.get();
        let error = if !has_sources {
            Some("No VIPER sources available.")
        } else if matches!(selection, SourceSelection::Left(_)) {
            Some("Select a VIPER source.")
        } else {
            None
        };

        if let Some(error) = error {
            let validated_selection = SourceSelection::Left(String::from(error));
            if selection != validated_selection {
                setter.set(validated_selection);
            }
        }
    });

    let viper_sources = Signal::derive(move || {
        let mut viper_sources = Clone::clone(&viper_sources);
        viper_sources.sort_by(|viper_source_a, viper_source_b| {
            viper_source_a.name.to_string().to_lowercase()
                .cmp(&viper_source_b.name.to_string().to_lowercase())
        });

        viper_sources.iter().map(|viper_source_descriptor| {
            let ViperSourceDescriptor { id, name, url, .. } = viper_source_descriptor;
            let id = id.to_owned();
            let name = name.to_string();
            let url = url.to_string();

            SelectionTableRow {
                id,
                cells: vec![name.into(), url.into()]
            }
        }).collect::<Vec<_>>()
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
