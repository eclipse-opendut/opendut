mod row;

use leptos::ev;
use leptos::prelude::*;
use leptos_use::{use_document, use_event_listener};
use opendut_lea_components::{has_text_selection, BasePageContainer, Breadcrumb, ButtonColor, ButtonSize, ButtonState, FontAwesomeIcon, IconButton, LoadingSpinner};
use opendut_model::viper::ViperTestDescriptor;
use crate::app::use_app_globals;
use crate::tests::components::CreateTestButton;
use crate::tests::overview::row::Row;

#[component(transparent)]
pub fn TestsOverview() -> impl IntoView {

    let globals = use_app_globals();
    let refetch_registered_tests = RwSignal::new(());

    let tests: LocalResource<Vec<ViperTestDescriptor>> = {
        let carl = globals.client.clone();

        LocalResource::new(move || {
            refetch_registered_tests.track();

            let mut carl = carl.clone();

            async move {
                let mut tests = carl.viper.list_viper_test_descriptors().await
                    .expect("Failed to request the list of tests / run descriptors.");

                tests.sort_by(|test_a, test_b| {
                    test_a.name.value().to_lowercase()
                        .cmp(&test_b.name.value().to_lowercase())
                });

                tests
            }
        })
    };

    let breadcrumbs = vec![
        Breadcrumb::new("Dashboard", "/"),
        Breadcrumb::new("Tests", "/tests")
    ];

    let block_row_click = RwSignal::new(false);
    let _ = use_event_listener(use_document(), ev::selectionchange, move |_| {
        block_row_click.set(has_text_selection());
    });

    view! {
        <BasePageContainer
            title="Tests"
            breadcrumbs
            controls=view! {
                <div class="buttons">
                    <CreateTestButton />
                    <IconButton
                        icon=FontAwesomeIcon::ArrowsRotate
                        color=ButtonColor::Light
                        size=ButtonSize::Normal
                        state=ButtonState::Enabled
                        label="Refresh table of peers"
                        on_action=move || {
                            refetch_registered_tests.notify();
                        }
                    />
                </div>
            }
        >
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
                    <Suspense
                        fallback=LoadingSpinner
                    >
                        { move || {
                            Suspend::new(async move {
                                let tests_table_rows = tests.await;

                                view! {
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
                                                    block_row_click
                                                    on_delete
                                                />
                                            }
                                        }}
                                    />
                                }
                            })
                        }}
                    </Suspense>
                </tbody>
            </table>
        </BasePageContainer>
    }
}
