mod row;

use leptos::prelude::*;
use opendut_lea_components::{BasePageContainer, Breadcrumb, ButtonColor, ButtonSize, ButtonState, FontAwesomeIcon, IconButton, OverviewTable, TableHeading};
use opendut_model::viper::ViperSourceDescriptor;
use crate::app::use_app_globals;
use crate::sources::components::CreateSourceButton;
use crate::sources::overview::row::Row;

#[component(transparent)]
pub fn SourcesOverview() -> impl IntoView {

    let globals = use_app_globals();

    let refetch_registered_sources = RwSignal::new(());

    let sources: LocalResource<Vec<ViperSourceDescriptor>> = {
        let carl = globals.client.clone();

        LocalResource::new(move || {
            refetch_registered_sources.track();

            let mut carl = carl.clone();

            async move {
                let mut sources = carl.viper.list_viper_source_descriptors().await
                    .expect("Failed to request the list of sources");

                sources.sort_by(|source_a, source_b| {
                    source_a.name.value().to_lowercase()
                        .cmp(&source_b.name.value().to_lowercase())
                });

                sources
            }
        })
    };

    let breadcrumbs = vec![
        Breadcrumb::new("Dashboard", "/"),
        Breadcrumb::new("Sources", "/sources")
    ];

    view! {
        <BasePageContainer
            title="Sources"
            breadcrumbs
            controls=view! {
                <div class="buttons">
                    <CreateSourceButton />
                    <IconButton
                        icon=FontAwesomeIcon::ArrowsRotate
                        color=ButtonColor::Light
                        size=ButtonSize::Normal
                        state=ButtonState::Enabled
                        label="Refresh table of peers"
                        on_action=move || {
                            refetch_registered_sources.notify();
                        }
                    />
                </div>
            }
        >
            { move || Suspend::new(async move {
                let sources = sources.await;

                let table_headings = vec![
                    TableHeading::new(String::from("Name")),
                    TableHeading::new(String::from("URL")),
                    TableHeading::new(String::from("Action")).set_narrow(),
                ];

                view! {
                    <OverviewTable headings=table_headings>
                        <For
                            each = move || sources.clone()
                            key = |source| source.id
                            children = { move |source_descriptor| {

                                let on_delete = move || {
                                    refetch_registered_sources.notify();
                                };

                                view! {
                                    <Row
                                        source_descriptor=RwSignal::new(source_descriptor)
                                        on_delete
                                    />
                                }
                            }}
                        />
                    </OverviewTable>
                }
            })}
        </BasePageContainer>
    }
}
