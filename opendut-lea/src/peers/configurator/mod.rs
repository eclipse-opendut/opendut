use crate::api::peers::get_peer_descriptor;
use crate::api::clusters::list_cluster_descriptors;
use crate::components::use_active_tab;
use crate::components::{BasePageContainer, Breadcrumb, UserInputError, UserInputValue};
use crate::peers::configurator::components::Controls;
use crate::peers::configurator::tabs::{DevicesTab,  GeneralTab, NetworkTab, SetupTab, TabIdentifier};
use crate::routing::{navigate_to, WellKnownRoutes};
use crate::util;
use leptos::either::Either;
use leptos::prelude::*;
use opendut_model::peer::PeerId;
use leptos_router::hooks::{use_navigate, use_params_map};
use opendut_lea_components::{ErrorMessage, LoadingSpinner};
use opendut_lea_components::tabs::{Tab, Tabs};
use crate::peers::configurator::types::network::UserPeerNetwork;
use crate::peers::configurator::types::UserPeerDescriptor;

mod components;
mod tabs;
mod types;

#[component(transparent)]
pub fn PeerConfigurator() -> impl IntoView {

    let params = use_params_map();
    let active_tab = use_active_tab::<TabIdentifier>();

    let peer_id = {
        let peer_id = params.with_untracked(|params| {
            params.get("id").and_then(|id| PeerId::try_from(id.as_str()).ok())
        });
        match peer_id {
            None => {
                let use_navigate = use_navigate();
                navigate_to(WellKnownRoutes::ErrorPage {
                    title: String::from("Invalid PeerId"),
                    text: String::from("Could not parse the provided value as PeerId!"),
                    details: None,
                }, use_navigate);
                PeerId::random()
            }
            Some(peer_id) => {
                peer_id
            }
        }
    };

    let user_peer_descriptor = RwSignal::new(UserPeerDescriptor {
        id: peer_id,
        name: UserInputValue::Left(UserInputError::from("Enter a valid peer name.")),
        location: UserInputValue::Right(String::from("")),
        devices: Vec::new(),
        network: UserPeerNetwork {
            network_interfaces: Vec::new(),
            bridge_name: UserInputValue::Right(String::from("")),
        },
        is_new: true,
    });

    let cluster_descriptors = list_cluster_descriptors(None);

    let peer_descriptor = get_peer_descriptor(Signal::from(peer_id), None);

    Effect::new(move |_| {
        if let (Some(Ok(Some(descriptor))), Some(Ok(clusters))) = (
            peer_descriptor.get(),
            cluster_descriptors.get(),
        ) {
            user_peer_descriptor.update(|configuration| configuration.apply_descriptor(descriptor, clusters));
        }
    });

    let peer_descriptor_state = Signal::derive(move || {
        match peer_descriptor.get() {
            Some(Ok(Some(_))) => cluster_descriptors.get().map(|result| result.map(|_| ())),
            Some(Ok(None)) => Some(Ok(())),
            Some(Err(error)) => Some(Err(error)),
            None => None,
        }
    });

    let peer_id_string = create_read_slice(user_peer_descriptor, |config| config.id.to_string());
    let setup_disabled = create_read_slice(user_peer_descriptor, |config| config.is_new);

    let cluster_column = move || {
        let peer = user_peer_descriptor.get();

        let devices_in_peer = peer.devices.into_iter()
            .map(|device| device.get().id)
            .collect::<Vec<_>>();

        let cluster_descriptor = user_peer_descriptor
            .get().devices
            .into_iter()
            .flat_map(|device| device.get().contained_in_clusters)
            .collect::<Vec<_>>();

        let configured_clusters = util::list_configured_clusters_for_peer(devices_in_peer, cluster_descriptor);

        let cluster_view_list: Vec<View<_>> = configured_clusters.into_iter()
            .map(|cluster| {
                let href = format!("/clusters/{}/configure/general", cluster.id);

                view! {
                    <a href=href>{cluster.name.to_string()}</a>
                }
            })
            .collect::<Vec<_>>();

        let amount_clusters = cluster_view_list.len();

        if amount_clusters > 0 {
            let comma_separated_cluster_views = util::view::join_with_comma_spans(cluster_view_list.clone());
            Either::Left(view! {
                <div class="mb-3">"Configured in " {amount_clusters} " clusters: " {comma_separated_cluster_views}</div>
            })
        } else {
            Either::Right(view! {
                <div class="mb-3">"Configured in " {amount_clusters} " clusters"</div>
            })
        }
    };

    let breadcrumbs = Signal::derive(move || {
        let peer_id = peer_id_string.get();
        let active_tab = active_tab.get();
        vec![
            Breadcrumb::new("Dashboard", "/"),
            Breadcrumb::new("Peers", "peers"),
            Breadcrumb::new("Configure Peer", format!("{}/configure/{}", peer_id, active_tab.as_str())),
        ]
    });

    let subtitle = Signal::derive(move || {
        if let UserInputValue::Right(name) = user_peer_descriptor.get().name {
            name
        } else {
            String::new()
        }
    });

    let tabs = Signal::derive(move || {

        let tabs = vec![
            Tab::from_title_and_href(
                String::from("General"),
                TabIdentifier::General.as_str().to_owned()
            ).with_is_error(Signal::derive(move || !user_peer_descriptor.read().valid_general_tab())),

            Tab::from_title_and_href(
                String::from("Network"),
                TabIdentifier::Network.as_str().to_owned()
            ).with_is_error(Signal::derive(move || !user_peer_descriptor.read().valid_network_tab())),

            Tab::from_title_and_href(
                String::from("Devices"),
                TabIdentifier::Devices.as_str().to_owned()
            ).with_is_error(Signal::derive(move || !user_peer_descriptor.read().valid_devices_tab())),

            Tab::from_title_and_href(
                String::from("Setup"),
                TabIdentifier::Setup.as_str().to_owned()
            ).with_is_hidden(setup_disabled),
        ];

        tabs
    });

    view! {
        <BasePageContainer
            title="Configure Peer"
            subtitle=subtitle
            breadcrumbs=breadcrumbs
            controls=view! { <Controls user_peer_descriptor /> }
        >
            <Show when=move || matches!(peer_descriptor_state.get(), Some(Ok(())))>
                <div> {cluster_column} </div>
            </Show>
            { move || {
                match peer_descriptor_state.get() {
                    Some(Ok(())) => {
                        view! {
                            <Tabs tabs active_tab=Signal::derive(move || active_tab.get().as_str())>
                                <div class=("is-hidden", move || TabIdentifier::General != active_tab.get())>
                                    <GeneralTab user_peer_descriptor />
                                </div>
                                <div class=("is-hidden", move || TabIdentifier::Network != active_tab.get())>
                                    <NetworkTab user_peer_descriptor />
                                </div>
                                <div class=("is-hidden", move || TabIdentifier::Devices != active_tab.get())>
                                    <DevicesTab user_peer_descriptor />
                                </div>
                                <div class=("is-hidden", move || TabIdentifier::Setup != active_tab.get())>
                                    <SetupTab peer_configuration=user_peer_descriptor.read_only() />
                                </div>
                            </Tabs>
                        }.into_any()
                    }
                    Some(Err(error)) => view! {
                        <ErrorMessage
                            text=error
                            on_retry=move || {
                                cluster_descriptors.refetch();
                                peer_descriptor.refetch();
                            }
                        />
                    }.into_any(),
                    None => view! { <LoadingSpinner /> }.into_any(),
                }
            }}
        </BasePageContainer>
    }
}
