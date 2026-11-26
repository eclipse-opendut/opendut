use crate::api::clusters::list_cluster_deployments;
use crate::util::client_error_description;
use leptos::prelude::*;
use leptos_router::hooks::{use_navigate, use_params_map};
use tracing::error;
use opendut_carl_api::carl::ClientError;
use opendut_carl_api::carl::cluster::GetClusterDescriptorError;
use opendut_lea_components::{ErrorMessage, ErrorText};
use opendut_lea_components::tabs::{Tab, Tabs};
use crate::components::UserInputValue;
use opendut_model::cluster::ClusterId;
use opendut_model::cluster::state::ClusterState;
use crate::app::use_app_globals;
use crate::clusters::configurator::components::{DeviceSelection, DeviceSelector, LeaderSelection};
use crate::clusters::configurator::components::Controls;
use crate::clusters::configurator::tabs::{DevicesTab, GeneralTab, LeaderTab, TabIdentifier};
use types::UserClusterDescriptor;
use crate::api::peers::list_peer_descriptors;
use crate::clusters::IsDeployed;
use crate::components::{use_active_tab, BasePageContainer, Breadcrumb, LoadingSpinner};
use crate::routing::{navigate_to, WellKnownRoutes};

mod tabs;
mod components;
mod types;

#[component(transparent)]
pub fn ClusterConfigurator() -> impl IntoView {
    let globals = use_app_globals();
    let params = use_params_map();
    let active_tab = use_active_tab::<TabIdentifier>();

    let cluster_id = {
        let cluster_id = params.with_untracked(|params| {
            params.get("id").and_then(|id| ClusterId::try_from(id.as_str()).ok())
        });
        match cluster_id {
            None => {
                let use_navigate = use_navigate();
                navigate_to(WellKnownRoutes::ErrorPage {
                    title: String::from("Invalid ClusterId"),
                    text: String::from("Could not parse the provided value as ClusterId!"),
                    details: None,
                }, use_navigate);
                ClusterId::random()
            }
            Some(cluster_id) => {
                cluster_id
            }
        }
    };

    let user_cluster_descriptor = RwSignal::new(
        UserClusterDescriptor {
            id: cluster_id,
            name: UserInputValue::Left(String::from("Enter a valid cluster name.")),
            devices: DeviceSelection::Left(String::from("Select at least two devices.")),
            leader: LeaderSelection::Left(String::from("Select a leader.")),
            is_new: true,
        }
    );

    let cluster_resource: LocalResource<Result<(), ErrorText>> = {
        let carl = globals.client.clone();

        LocalResource::new(move || {
            let mut carl = carl.clone();

            async move {
                let descriptor = match carl.cluster.get_cluster_descriptor(cluster_id).await {
                    Ok(descriptor) => descriptor,
                    Err(ClientError::UsageError(GetClusterDescriptorError::ClusterNotFound { .. })) => {
                        return Ok(());
                    }
                    Err(error) => {
                        error!("Failed to request the cluster descriptor for cluster <{cluster_id}>: {error}");

                        let error_description = client_error_description(
                            &error,
                            "The cluster descriptor could not be loaded. Please check your connection and try again.",
                        );

                        return Err(ErrorText {
                            headline: String::from("Unable to load cluster descriptor"),
                            description: error_description,
                        })
                    }
                };

                user_cluster_descriptor.set(
                    UserClusterDescriptor {
                        id: cluster_id,
                        name: UserInputValue::Right(descriptor.name.value().to_owned()),
                        devices: DeviceSelection::Right(descriptor.devices),
                        leader: LeaderSelection::Right(descriptor.leader),
                        is_new: false,
                    }
                );

                Ok(())
            }
        })
    };

    let cluster_state = RwSignal::new(ClusterState::default());

    let peers_resource = list_peer_descriptors(None);

    let breadcrumbs = Signal::derive(move || {
        let cluster_id = cluster_id.uuid.to_string();
        let active_tab = active_tab.get();
        vec![
            Breadcrumb::new("Dashboard", "/"),
            Breadcrumb::new("Clusters", "clusters"),
            Breadcrumb::new("Configure Cluster", format!("{}/configure/{}", cluster_id, active_tab.as_str())),
        ]
    });

    let refetch_cluster_deployments = RwSignal::new(());

    let cluster_deployments = list_cluster_deployments(Some(refetch_cluster_deployments));

    let deployed_signal = Signal::derive(move || {
        let cluster_id = cluster_id;

        match cluster_deployments.get() {
            Some(Ok(deployments)) => {
                Some(IsDeployed(deployments.iter().any(|deployment| deployment.id == cluster_id)))
            }
            Some(Err(_)) | None => None
        }
    });

    let subtitle = Signal::derive(move || {
        if let UserInputValue::Right(name) = user_cluster_descriptor.get().name {
            name
        } else {
            String::new()
        }
    });

    let tabs = Signal::derive(move || {
        vec![
            Tab::from_title_and_href(
                String::from("General"),
                TabIdentifier::General.as_str().to_owned()
            ).with_is_error(Signal::derive(move || !user_cluster_descriptor.read().valid_general_tab())),

            Tab::from_title_and_href(
                String::from("Devices"),
                TabIdentifier::Devices.as_str().to_owned()
            ).with_is_error(Signal::derive(move || !user_cluster_descriptor.read().valid_devices_tab())),

            Tab::from_title_and_href(
                String::from("Leader"),
                TabIdentifier::Leader.as_str().to_owned()
            ).with_is_error(Signal::derive(move || !user_cluster_descriptor.read().valid_leader_tab())),
        ]
    });

    let is_disabled = Signal::derive(move || {
        !matches!(deployed_signal.get(), Some(IsDeployed(false)))
    });

    view! {
        <BasePageContainer
            title="Configure Cluster"
            subtitle
            breadcrumbs
            controls=move || {
                view! {
                    <Controls
                        user_cluster_descriptor
                        deployed_signal
                        cluster_state=cluster_state.into()
                        on_deployment_change=move || refetch_cluster_deployments.notify()
                    />
                }
            }
        >
            <Suspense
                fallback=move || view! { <LoadingSpinner /> }
            >
                { move || Suspend::new(async move {
                    let peers = RwSignal::new(peers_resource.await).read_only();
                    match cluster_resource.await {
                        Ok(()) => {
                            view! {
                                <Tabs tabs active_tab=Signal::derive(move || active_tab.get().as_str())>
                                    { move || match active_tab.get() {
                                        TabIdentifier::General => view! {
                                            <fieldset prop:disabled=move || is_disabled.get()>
                                                <GeneralTab user_cluster_descriptor />
                                            </fieldset>
                                        }.into_any(),

                                        TabIdentifier::Devices => view! {
                                            <DevicesTab
                                                user_cluster_descriptor
                                                peers=peers
                                                on_retry=move || peers_resource.refetch()
                                                is_disabled
                                            />
                                        }.into_any(),

                                        TabIdentifier::Leader => view! {
                                            <LeaderTab
                                                user_cluster_descriptor
                                                peers=peers
                                                on_retry=move || peers_resource.refetch()
                                                is_disabled
                                            />
                                        }.into_any()
                                    }}
                                </Tabs>
                            }.into_any()
                        }
                        Err(error) => view! {
                            <ErrorMessage
                                text=error
                                on_retry=move || cluster_resource.refetch()
                            />
                        }.into_any(),
                    }
                })}
            </Suspense>
        </BasePageContainer>
    }
}
