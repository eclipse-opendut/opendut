mod row;

use leptos::prelude::*;
use opendut_lea_components::{BasePageContainer, Breadcrumb, ButtonColor, ButtonSize, ButtonState, FontAwesomeIcon, IconButton, OverviewTable, TableHeading};
use opendut_model::viper::ViperTestRunDescriptor;
use crate::app::use_app_globals;
use crate::viper_tests::components::CreateViperTestButton;
use crate::viper_tests::overview::row::Row;

#[component(transparent)]
pub fn ViperTestsOverview() -> impl IntoView {

    let globals = use_app_globals();
    let refetch_viper_tests = RwSignal::new(());

    let viper_tests: LocalResource<Vec<ViperTestRunDescriptor>> = {
        let carl = globals.client.clone();

        LocalResource::new(move || {
            refetch_viper_tests.track();

            let mut carl = carl.clone();

            async move {
                let mut viper_tests = carl.viper.list_viper_test_run_descriptors().await
                    .expect("Failed to request the list of viper test descriptors.");

                viper_tests.sort_by(|viper_test_a, viper_test_b| {
                    viper_test_a.name.value().to_lowercase()
                        .cmp(&viper_test_b.name.value().to_lowercase())
                });

                viper_tests
            }
        })
    };

    let breadcrumbs = vec![
        Breadcrumb::new("Dashboard", "/"),
        Breadcrumb::new("Viper Tests", "/viper_tests")
    ];

    let table_headings = vec![
        TableHeading::new(String::from("Name")),
        TableHeading::new(String::from("Viper Source")),
        TableHeading::new(String::from("Action")).set_narrow(),
    ];

    view! {
        <BasePageContainer
            title="Viper Tests"
            breadcrumbs
            controls=view! {
                <div class="buttons">
                    <CreateViperTestButton />
                    <IconButton
                        icon=FontAwesomeIcon::ArrowsRotate
                        color=ButtonColor::Light
                        size=ButtonSize::Normal
                        state=ButtonState::Enabled
                        label="Refresh table of viper tests"
                        on_action=move || {
                            refetch_viper_tests.notify();
                        }
                    />
                </div>
            }
        >
        <OverviewTable headings=table_headings>
            { move || Suspend::new(async move {
                    let viper_tests = viper_tests.await;
                    view! {
                        <For
                            each = move || viper_tests.clone()
                            key = |viper_tests| viper_tests.id
                            children = { move |viper_test_run_descriptor| {

                                let on_delete = move || {
                                    refetch_viper_tests.notify();
                                };

                                view! {
                                    <Row
                                        viper_test_run_descriptor=RwSignal::new(viper_test_run_descriptor)
                                        on_delete
                                    />
                                }
                            }}
                        />
                    }
                })
            }
            </OverviewTable>
        </BasePageContainer>
    }
}
