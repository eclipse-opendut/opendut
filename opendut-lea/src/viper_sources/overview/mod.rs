mod row;

use leptos::prelude::*;
use opendut_lea_components::{BasePageContainer, Breadcrumb, ButtonColor, ButtonSize, ButtonState, FontAwesomeIcon, IconButton, OverviewTable, TableHeading};
use opendut_model::viper::ViperSourceDescriptor;
use crate::app::use_app_globals;
use crate::viper_sources::components::CreateViperSourceButton;
use crate::viper_sources::overview::row::Row;

#[component(transparent)]
pub fn ViperSourcesOverview() -> impl IntoView {

    let globals = use_app_globals();

    let refetch_viper_sources = RwSignal::new(());

    let viper_sources: LocalResource<Vec<ViperSourceDescriptor>> = {
        let carl = globals.client.clone();

        LocalResource::new(move || {
            refetch_viper_sources.track();

            let mut carl = carl.clone();

            async move {
                let mut viper_sources = carl.viper.list_viper_source_descriptors().await
                    .expect("Failed to request the list of viper_sources");

                viper_sources.sort_by(|viper_source_a, viper_source_b| {
                    viper_source_a.name.value().to_lowercase()
                        .cmp(&viper_source_b.name.value().to_lowercase())
                });

                viper_sources
            }
        })
    };

    let breadcrumbs = vec![
        Breadcrumb::new("Dashboard", "/"),
        Breadcrumb::new("Viper Sources", "/viper_sources")
    ];

    let table_headings = vec![
        TableHeading::new(String::from("Name")),
        TableHeading::new(String::from("URL")),
        TableHeading::new(String::from("Action")).set_narrow(),
    ];

    view! {
        <BasePageContainer
            title="Sources"
            breadcrumbs
            controls=view! {
                <div class="buttons">
                    <CreateViperSourceButton />
                    <IconButton
                        icon=FontAwesomeIcon::ArrowsRotate
                        color=ButtonColor::Light
                        size=ButtonSize::Normal
                        state=ButtonState::Enabled
                        label="Refresh table of viper_sources"
                        on_action=move || {
                            refetch_viper_sources.notify();
                        }
                    />
                </div>
            }
        >
            <OverviewTable headings=table_headings>
                { move || Suspend::new(async move {
                    let viper_sources = viper_sources.await;
                    view! {
                        <For
                            each = move || viper_sources.clone()
                            key = |viper_source| viper_source.id
                            children = { move |viper_source_descriptor| {

                                let on_delete = move || {
                                    refetch_viper_sources.notify();
                                };

                                view! {
                                    <Row
                                        viper_source_descriptor=RwSignal::new(viper_source_descriptor)
                                        on_delete
                                    />
                                }
                            }}
                        />
                    }
                })}
            </OverviewTable>
        </BasePageContainer>
    }
}
