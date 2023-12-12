use leptos::*;
use leptos_router::use_params_map;

use opendut_types::peer::PeerId;

use crate::api::use_carl;
use crate::components::{BasePageContainer, Breadcrumb, UserInputError, UserInputValue};
use crate::components::use_active_tab;
use crate::peers::configurator::components::Controls;
use crate::peers::configurator::tabs::{DevicesTab, GeneralTab, SetupTab, TabIdentifier};
use crate::peers::configurator::types::UserPeerConfiguration;
use crate::routing::{navigate_to, WellKnownRoutes};

mod components;
mod tabs;
mod types;

#[component]
pub fn PeerConfigurator() -> impl IntoView {

    let carl = use_carl();
    let params = use_params_map();

    let active_tab = use_active_tab::<TabIdentifier>();

    let peer_configuration = {

        let peer_id = {
            let peer_id = params.with_untracked(|params| {
                params.get("id").and_then(|id| PeerId::try_from(id.as_str()).ok())
            });
            match peer_id {
                None => {
                    navigate_to(WellKnownRoutes::ErrorPage {
                        title: String::from("Invalid PeerId"),
                        text: String::from("Could not parse the provided value as PeerId!"),
                        details: None,
                    });

                    PeerId::default()
                }
                Some(peer_id) => {
                    peer_id
                }
            }
        };

        let peer_configuration = create_rw_signal(UserPeerConfiguration {
            id: peer_id,
            name: UserInputValue::Left(UserInputError::from("Enter a valid peer name.")),
            is_new: true,
        });

        create_local_resource(|| {}, move |_| {
            let mut carl = carl.get_untracked();
            async move {
                if let Ok(configuration) = carl.peers.get_peer(peer_id).await {
                    peer_configuration.update(|user_configuration| {
                        user_configuration.name = UserInputValue::Right(configuration.name.value());
                        user_configuration.is_new = false;
                    });
                }
            }
        });

        peer_configuration
    };

    let peer_id_string = create_read_slice(peer_configuration, |config| config.id.to_string());
    let setup_disabled = create_read_slice(peer_configuration, |config| config.is_new);

    let setup_tab_classes = move || {
        let mut classes = Vec::<&'static str>::new();
        if TabIdentifier::Setup == active_tab.get() {
            classes.push("is-active");
        }
        if setup_disabled.get() {
            classes.push("is-hidden");
        }
        classes.join(" ")
    };

    let breadcrumbs = {
        let active_tab = Clone::clone(&active_tab);
        MaybeSignal::derive(move || {
            let peer_id = peer_id_string.get();
            let active_tab = active_tab.get();
            vec![
                Breadcrumb::new("Dashboard", "/"),
                Breadcrumb::new("Peers", "peers"),
                Breadcrumb::new(Clone::clone(&peer_id), format!("{}/configure/{}", peer_id, active_tab.to_str())),
            ]
        })
    };

    view! {
        <BasePageContainer
            title="Configure Peer"
            breadcrumbs=breadcrumbs
            controls=view! { <Controls configuration=peer_configuration.read_only() /> }
        >
            <div>
                <div class="tabs">
                    <ul>
                        <li class=("is-active", move || TabIdentifier::General == active_tab.get())>
                            <a href={ TabIdentifier::General.to_str() }>
                                <span>General</span>
                                // An icon could indicate a misconfiguration on a tabs
                                // <span class="icon is-small has-text-danger"><i class="fa-solid fa-circle-exclamation" aria-hidden="true"></i></span>
                            </a>
                        </li>
                        // <li class=("is-active", move || TabIdentifier::Devices == active_tab.get())>
                        //     <a href={ TabIdentifier::Devices.to_str() }>Devices</a>
                        // </li>
                        <li class=setup_tab_classes>
                            <a href={ TabIdentifier::Setup.to_str() }>Setup</a>
                        </li>
                    </ul>
                </div>
                <div class="container">
                    <div class=("is-hidden", move || TabIdentifier::General != active_tab.get())>
                        <GeneralTab peer_configuration=peer_configuration />
                    </div>
                    <div class=("is-hidden", move || TabIdentifier::Devices != active_tab.get())>
                        <DevicesTab peer_configuration=peer_configuration />
                    </div>
                    <div class=("is-hidden", move || TabIdentifier::Setup != active_tab.get())>
                        <SetupTab peer_configuration=peer_configuration.read_only() />
                    </div>
                </div>
            </div>
        </BasePageContainer>
    }
}
