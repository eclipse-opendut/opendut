mod row;

use leptos::prelude::*;
use opendut_lea_components::{BasePageContainer, Breadcrumb, ButtonColor, ButtonSize, ButtonState, FontAwesomeIcon, IconButton, OverviewTable, TableHeading};
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

    let table_headings = vec![
        TableHeading::new(String::from("Name")),
        TableHeading::new(String::from("Source")),
        TableHeading::new(String::from("Suite")),
        TableHeading::new(String::from("Action")).set_narrow(),
    ];

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
                        label="Refresh table of tests"
                        on_action=move || {
                            refetch_registered_tests.notify();
                        }
                    />
                </div>
            }
        >
        <OverviewTable headings=table_headings>
            { move || Suspend::new(async move {
                    let tests = tests.await;
                    view! {
                        <For
                            each = move || tests.clone()
                            key = |tests| tests.id
                            children = { move |tests_descriptor| {

                                let on_delete = move || {
                                    refetch_registered_tests.notify();
                                };

                                view! {
                                    <Row
                                        test_descriptor=RwSignal::new(tests_descriptor)
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
