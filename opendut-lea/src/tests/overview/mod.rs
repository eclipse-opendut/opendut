mod row;

use leptos::prelude::*;
use opendut_lea_components::{BasePageContainer, Breadcrumb, LoadingSpinner};
use opendut_model::viper::ViperRunDescriptor;
use crate::app::use_app_globals;
use crate::tests::overview::row::Row;

#[component(transparent)]
pub fn TestsOverview() -> impl IntoView {

    let globals = use_app_globals();
    let refetch_registered_tests = RwSignal::new(());

    let registered_tests: LocalResource<Vec<ViperRunDescriptor>> = {
        let carl = globals.client.clone();

        LocalResource::new(move || {
            refetch_registered_tests.track();

            let mut carl = carl.clone();

            async move {
                carl.viper.list_viper_run_descriptors().await
                    .expect("Failed to request the list of tests / run descriptors.")
            }
        })
    };

    let tests_table_rows = LocalResource::new(move || async move {
        let registered_tests = registered_tests.await;

        // Todo: Sort registered_tests.

        registered_tests
    });

    let breadcrumbs = vec![
        Breadcrumb::new("Dashboard", "/"),
        Breadcrumb::new("Tests", "/tests")
    ];

    view! {
        <BasePageContainer
            title="Tests"
            breadcrumbs
            controls=view! {} // Todo: CreateTestButton
        >
        <div class="mt-4">
                <Transition
                    fallback=LoadingSpinner
                >
                    {move || {
                        Suspend::new(async move {
                            let tests_table_rows = tests_table_rows.await;

                            view! {
                                <table class="table is-hoverable is-fullwidth">
                                    <thead>
                                        <tr>
                                            <th>"Name"</th>
                                            <th>"Source"</th>
                                            <th>"Suite"</th>
                                            <th class="is-narrow has-text-centered">"Action"</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        <For
                                            each = move || tests_table_rows.clone()
                                            key = |test| test.id
                                            children = { move |test_descriptor| {
                                                let on_delete = move || {
                                                    refetch_registered_tests.notify();
                                                };
                                                view! {
                                                    <Row
                                                        test_descriptor=RwSignal::new(test_descriptor)
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
