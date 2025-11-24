mod row;

use leptos::prelude::*;
use opendut_lea_components::{BasePageContainer, Breadcrumb, LoadingSpinner};
use opendut_model::viper::ViperSourceDescriptor;
use crate::app::use_app_globals;
use crate::sources::components::CreateSourceButton;
use crate::sources::overview::row::Row;

#[component(transparent)]
pub fn SourcesOverview() -> impl IntoView {

    let globals = use_app_globals();

    let refetch_registered_sources = RwSignal::new(());

    let registered_sources: LocalResource<Vec<ViperSourceDescriptor>> = {
        let carl = globals.client.clone();

        LocalResource::new(move || {
            refetch_registered_sources.track();

            let mut carl = carl.clone();

            async move {
                carl.viper.list_viper_source_descriptors().await
                    .expect("Failed to request the list of sources")
            }
        })
    };

    let sources_table_rows = LocalResource::new(move || async move {
        let mut registered_sources = registered_sources.await;
        registered_sources.sort_by(|source_a, source_b| {
            source_a.name.value().to_lowercase()
                .cmp(&source_b.name.value().to_lowercase())
        });

        registered_sources
    });

    let breadcrumbs = vec![
        Breadcrumb::new("Dashboard", "/"),
        Breadcrumb::new("Sources", "/sources")
    ];

    view! {
        <BasePageContainer
            title="Sources"
            breadcrumbs=breadcrumbs
            controls=view! {
                <CreateSourceButton />
            }
        >

        <div class="mt-4">
                <Transition
                    fallback=LoadingSpinner
                >
                    {move || {
                        Suspend::new(async move {
                            let sources_table_rows = sources_table_rows.await;

                            view! {
                                <table class="table is-hoverable is-fullwidth">
                                    <thead>
                                        <tr>
                                            <th>"Name"</th>
                                            <th>"Url"</th>
                                            <th class="is-narrow has-text-centered">"Action"</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        <For
                                            each = move || sources_table_rows.clone()
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
                                    </tbody>
                                </table>
                            }
                        })
                    }}
                </Transition>
            </div>
        </BasePageContainer>
    }
}
