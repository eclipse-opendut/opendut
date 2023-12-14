use leptos::*;
use leptos_router::use_params_map;

use opendut_types::cluster::ClusterId;

use crate::app::{ExpectGlobals, use_app_globals};
use crate::clusters::configurator::components::{DeviceSelection, DeviceSelector};
use crate::clusters::configurator::components::Controls;
use crate::clusters::configurator::tabs::{DevicesTab, GeneralTab, TabIdentifier};
use crate::clusters::configurator::types::UserClusterConfiguration;
use crate::components::{BasePageContainer, Breadcrumb, Initialized, use_active_tab};
use crate::components::{UserInputError, UserInputValue};
use crate::routing::{navigate_to, WellKnownRoutes};

mod types;
mod tabs;
mod components;

#[component(transparent)]
pub fn ClusterConfigurator() -> impl IntoView {

    #[component]
    fn inner() -> impl IntoView {

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
                        navigate_to(WellKnownRoutes::ErrorPage {
                            title: String::from("Invalid ClusterId"),
                            text: String::from("Could not parse the provided value as ClusterId!"),
                            details: None,
                        });

                        ClusterId::default()
                    }
                    Some(cluster_id) => {
                        cluster_id
                    }
                }
            };

            let user_configuration = create_rw_signal(UserClusterConfiguration {
                id: cluster_id,
                name: UserInputValue::Left(UserInputError::from("Enter a valid cluster name.")),
                devices: DeviceSelection::Left(String::from("Select at least two devices.")),
            });

            create_local_resource(|| {}, move |_| { // TODO: maybe a action suits better here
                let mut carl = globals.expect_client();
                async move {
                    if let Ok(configuration) = carl.cluster.get_cluster_configuration(cluster_id).await {
                        user_configuration.update(|user_configuration| {
                            user_configuration.name = UserInputValue::Right(configuration.name.value());
                            user_configuration.devices = DeviceSelection::Right(configuration.devices);
                        });
                    }
                }
            });

            user_configuration
        };

        let cluster_id = create_read_slice(cluster_configuration, |config| config.id.to_string());

        let breadcrumbs = MaybeSignal::derive(move || {
            let cluster_id = cluster_id.get();
            vec![
                Breadcrumb::new("Dashboard", "/"),
                Breadcrumb::new("Clusters", "clusters"),
                Breadcrumb::new(Clone::clone(&cluster_id), cluster_id),
            ]
        });

        view! {
            <BasePageContainer
                title="Configure Cluster"
                breadcrumbs=breadcrumbs
                controls=view! { <Controls cluster_configuration=cluster_configuration.read_only() /> }
            >
                <div>

                    <div class="tabs">
                        <ul>
                            <li class=("is-active", move || TabIdentifier::General == active_tab.get())>
                                <a href={ TabIdentifier::General.to_str() }>General</a>
                            </li>
                            <li class=("is-active", move || TabIdentifier::Devices == active_tab.get())>
                                <a href={ TabIdentifier::Devices.to_str() }>Devices</a>
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
                    </div>
                </div>
            </BasePageContainer>
        }
    }

    view! {
        <Initialized>
            <Inner />
        </Initialized>
    }
}
