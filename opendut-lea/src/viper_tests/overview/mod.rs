use crate::util::client_error_description;
mod row;

use leptos::prelude::*;
use tracing::error;
use opendut_lea_components::{BasePageContainer, Breadcrumb, ButtonColor, ButtonSize, ButtonState, ErrorMessage, ErrorMessageType, ErrorMessageTableRow, ErrorText, FontAwesomeIcon, IconButton, InfoMessage, OverviewTable, TableHeading};
use opendut_model::peer::PeerDescriptor;
use opendut_model::viper::{ViperSourceDescriptor, ViperTestRunDescriptor};
use crate::app::use_app_globals;
use crate::viper_tests::components::CreateViperTestButton;
use crate::viper_tests::overview::row::Row;

#[component(transparent)]
pub fn ViperTestsOverview() -> impl IntoView {

    let globals = use_app_globals();
    let refetch_viper_tests = RwSignal::new(());

    let viper_tests: LocalResource<Result<Vec<ViperTestRunDescriptor>, ErrorText>> = {
        let carl = globals.client.clone();

        LocalResource::new(move || {
            refetch_viper_tests.track();

            let mut carl = carl.clone();

            async move {
                let mut viper_tests = carl.viper.list_viper_test_run_descriptors().await
                    .map_err(|error| {
                        error!("Failed to request the list of VIPER test descriptors: {error}");

                        let error_description = client_error_description(
                            &error,
                            "The VIPER test descriptors could not be loaded. Please check your connection and try again.",
                        );

                        ErrorText {
                            headline: String::from("Unable to load VIPER test descriptors"),
                            description: error_description,
                        }
                    })?;

                viper_tests.sort_by(|viper_test_a, viper_test_b| {
                    viper_test_a.name.value().to_lowercase()
                        .cmp(&viper_test_b.name.value().to_lowercase())
                });

                Ok(viper_tests)
            }
        })
    };

    let viper_sources: LocalResource<Result<Vec<ViperSourceDescriptor>, ErrorText>> = {
        let carl = globals.client.clone();

        LocalResource::new(move || {
            refetch_viper_tests.track();

            let mut carl = carl.clone();
            async move {
                carl.viper.list_viper_source_descriptors().await
                    .map_err(|error| {
                        error!("Failed to request the list of VIPER sources: {error}");

                        let error_description = client_error_description(
                            &error,
                            "The VIPER sources could not be loaded. Please check your connection and try again.",
                        );

                        ErrorText {
                            headline: String::from("Unable to load VIPER sources"),
                            description: error_description,
                        }
                    })
            }
        })
    };

    let peers: LocalResource<Result<Vec<PeerDescriptor>, ErrorText>> = {
        let carl = globals.client.clone();

        LocalResource::new(move || {
            refetch_viper_tests.track();

            let mut carl = carl.clone();
            async move {
                carl.peers.list_peer_descriptors().await
                    .map_err(|error| {
                        error!("Failed to request the list of peers: {error}");

                        let error_description = client_error_description(
                            &error,
                            "The peer descriptors could not be loaded. Please check your connection and try again.",
                        );

                        ErrorText {
                            headline: String::from("Unable to load peer descriptors"),
                            description: error_description,
                        }
                    })
            }
        })
    };

    let breadcrumbs = vec![
        Breadcrumb::new("Dashboard", "/"),
        Breadcrumb::new("VIPER Tests", "/viper_tests")
    ];

    let table_headings = vec![
        TableHeading::new(String::from("Deploy")).set_narrow(),
        TableHeading::new(String::from("Name")),
        TableHeading::new(String::from("VIPER Source")),
        TableHeading::new(String::from("Run on Peer")),
        TableHeading::new(String::from("Action")).set_narrow(),
    ];

    view! {
        <BasePageContainer
            title="VIPER Tests"
            breadcrumbs
            controls=view! {
                <div class="buttons">
                    <CreateViperTestButton />
                    <IconButton
                        icon=FontAwesomeIcon::ArrowsRotate
                        color=ButtonColor::Light
                        size=ButtonSize::Normal
                        state=ButtonState::Enabled
                        label="Refresh table of VIPER tests"
                        on_action=move || {
                            refetch_viper_tests.notify();
                        }
                    />
                </div>
            }
        >
        <InfoMessage
            text="A VIPER Test contains the concrete configuration for how and where your test code should be executed."
        />
            <Transition fallback=|| ()>
                { move || match viper_sources.get() {
                    Some(Err(error)) => view! {
                        <ErrorMessage
                            text=error
                            on_retry=move || viper_sources.refetch()
                            error_type=ErrorMessageType::Warning
                        />
                    }.into_any(),
                    Some(Ok(_)) | None => ().into_any(),
                }}
                { move || match peers.get() {
                    Some(Err(error)) => view! {
                        <ErrorMessage
                            text=error
                            on_retry=move || peers.refetch()
                            error_type=ErrorMessageType::Warning
                        />
                    }.into_any(),
                    Some(Ok(_)) | None => ().into_any(),
                }}
            </Transition>
            <OverviewTable headings=table_headings>
                { move || Suspend::new(async move {
                    let viper_tests_result = viper_tests.await;

                    match viper_tests_result {
                        Ok(viper_tests) => {
                            view! {
                                <ViperTestsOverviewTable
                                    viper_tests
                                    viper_sources
                                    peers
                                    refetch_viper_tests
                                />
                            }.into_any()
                        }
                        Err(error) => {
                            view! {
                                <ErrorMessageTableRow
                                    text=error
                                    on_retry=move || viper_tests.refetch()
                                />
                            }.into_any()
                        }
                    }
                })}
            </OverviewTable>
        </BasePageContainer>
    }
}

#[component]
fn ViperTestsOverviewTable(
    viper_tests: Vec<ViperTestRunDescriptor>,
    viper_sources: LocalResource<Result<Vec<ViperSourceDescriptor>, ErrorText>>,
    peers: LocalResource<Result<Vec<PeerDescriptor>, ErrorText>>,
    refetch_viper_tests: RwSignal<()>,
) -> impl IntoView {

    let on_delete = move || {
        refetch_viper_tests.notify();
    };

    if viper_tests.is_empty() {
        view! {
            <tr>
                <td colspan="999" class="has-text-centered">"No VIPER tests have been created yet"</td>
            </tr>
        }.into_any()
    } else {
        view! {
            <For
                each = move || viper_tests.clone()
                key = |viper_test| viper_test.id
                children = { move |viper_test_run_descriptor| {
                    view! {
                        <Row
                            viper_test_run_descriptor=RwSignal::new(viper_test_run_descriptor)
                            viper_sources
                            peers
                            refetch_viper_tests
                            on_delete
                        />
                    }
                }}
            />
        }.into_any()
    }
}
