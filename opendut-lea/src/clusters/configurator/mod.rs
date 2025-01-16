use leptos::prelude::*;
use leptos_router::hooks::{use_navigate, use_params_map};
use opendut_types::cluster::ClusterId;

use crate::app::use_app_globals;
use crate::clusters::configurator::components::{DeviceSelection, DeviceSelector, LeaderSelection};
use crate::clusters::configurator::components::Controls;
use crate::clusters::configurator::tabs::{DevicesTab, GeneralTab, LeaderTab, TabIdentifier};
use crate::clusters::configurator::types::UserClusterConfiguration;
use crate::clusters::overview::IsDeployed;
use crate::components::{BasePageContainer, Breadcrumb, use_active_tab};
use crate::components::{UserInputError, UserInputValue};
use crate::routing::{navigate_to, WellKnownRoutes};

mod types;
mod tabs;
mod components;

#[component(transparent)]
pub fn ClusterConfigurator() -> impl IntoView {

    let globals = use_app_globals();
    let params = use_params_map();

    let active_tab = use_active_tab::<TabIdentifier>();

    let cluster_configuration = {
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

                    ClusterId::default()
                }
                Some(cluster_id) => {
                    cluster_id
                }
            }
        };

        let user_configuration = RwSignal::new(UserClusterConfiguration {
            id: cluster_id,
            name: UserInputValue::Left(UserInputError::from("Enter a valid cluster name.")),
            devices: DeviceSelection::Left(String::from("Select at least two devices.")),
            leader: LeaderSelection::Left(String::from("Select a leader.")),
        });

        let carl = globals.client.clone();

        LocalResource::new(move || {
            let mut carl = carl.clone();
            async move {
                if let Ok(configuration) = carl.cluster.get_cluster_configuration(cluster_id).await {
                    user_configuration.update(|user_configuration| {
                        user_configuration.name = UserInputValue::Right(configuration.name.value());
                        user_configuration.devices = DeviceSelection::Right(configuration.devices);
                        user_configuration.leader = LeaderSelection::Right(configuration.leader);
                    });
                }
            }
        });

        user_configuration
    };

    let cluster_id = create_read_slice(cluster_configuration, |config| config.id);

    let breadcrumbs = Signal::derive(move || {
        let cluster_id = cluster_id.get().0.to_string();
        vec![
            Breadcrumb::new("Dashboard", "/"),
            Breadcrumb::new("Clusters", "clusters"),
            Breadcrumb::new(Clone::clone(&cluster_id), cluster_id),
        ]
    });

    let cluster_deployments = {
        let carl = globals.client.clone();
        LocalResource::new(move || {
            let mut carl = carl.clone();
            async move {
                carl.cluster.list_cluster_deployments().await
                    .expect("Failed to request the list of cluster deployments")
            }
        })
    };


    let deployed_clusters = move || {
        match cluster_deployments.get() {
            Some(deployed_clusters) => {
                deployed_clusters.iter().map(|cluster_deployment| cluster_deployment.id).collect::<Vec<_>>()
            }
            None => Vec::new()
        }
    };

    let deployed_rw_signal = move || {
        RwSignal::new(IsDeployed(deployed_clusters().contains(&cluster_id.get())))
    };

    let is_deployed = move || {
        Signal::derive(move || {
            IsDeployed(deployed_clusters().contains(&cluster_id.get())).0
        })
    };

    view! {
        <BasePageContainer
            title="Configure Cluster"
            breadcrumbs=breadcrumbs
            controls=move || {
                view! {
                    <Controls
                        cluster_configuration=cluster_configuration.read_only()
                        deployed_signal=deployed_rw_signal()
                    />
                }
            }
        >
            <fieldset prop:disabled=is_deployed()>

                <div class="tabs">
                    <ul>
                        <li class=("is-active", move || TabIdentifier::General == active_tab.get())>
                            <a href={ TabIdentifier::General.as_str() }>General</a>
                        </li>
                        <li class=("is-active", move || TabIdentifier::Devices == active_tab.get())>
                            <a href={ TabIdentifier::Devices.as_str() }>Devices</a>
                        </li>
                        <li class=("is-active", move || TabIdentifier::Leader == active_tab.get())>
                            <a href={ TabIdentifier::Leader.as_str() }>Leader</a>
                        </li>
                    </ul>
                </div>
                <div class="container">
                    <div class=("is-hidden", move || TabIdentifier::General != active_tab.get())>
                        <GeneralTab cluster_configuration=cluster_configuration />
                    </div>
                    <div class=("is-hidden", move || TabIdentifier::Devices != active_tab.get())>
                        <DevicesTab cluster_configuration=cluster_configuration />
                    </div>
                    <div class=("is-hidden", move || TabIdentifier::Leader != active_tab.get())>
                        <LeaderTab cluster_configuration=cluster_configuration />
                    </div>
                </div>
            </fieldset>
        </BasePageContainer>
    }
}
