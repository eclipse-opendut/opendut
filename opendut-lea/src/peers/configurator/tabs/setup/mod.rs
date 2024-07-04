use leptos::{component, create_local_resource, IntoView, ReadSignal, RwSignal, SignalGet, SignalSet, use_context, view, WriteSignal};
use opendut_auth::public::OptionalAuthData;

use opendut_types::peer::PeerId;

use crate::app::{ExpectGlobals, use_app_globals};
use crate::components::{ButtonColor, SimpleButton};
use crate::components::ButtonStateSignalProvider;
use crate::peers::configurator::types::UserPeerConfiguration;

#[component]
pub fn SetupTab(peer_configuration: ReadSignal<UserPeerConfiguration>) -> impl IntoView {

    let globals = use_app_globals();

    let trigger_generation: RwSignal<Option<PeerId>> = RwSignal::new(None);

    let (auth_data_signal, _) = use_context::<(ReadSignal<OptionalAuthData>, WriteSignal<OptionalAuthData>)>().expect("AuthData should be provided in the context.");

    let setup_string = create_local_resource(move || trigger_generation.get(), move |peer_id| {
        async move {
            let user_id = match auth_data_signal.get().auth_data {
                None => { String::from("UNKNOWN USER") }
                Some(auth_data) => { auth_data.subject }
            };
            if let Some(peer_id) = peer_id {
                let mut carl = globals.expect_client();
                let setup = carl.peers.create_peer_setup(peer_id, user_id).await
                    .expect("Failed to request the setup string.");
                let setup_string = setup.encode()
                    .expect("PeerSetup should be encodable into a setup-string");
                Some(setup_string)
            }
            else {
                None
            }
        }
    });

    let button_state = setup_string.loading().derive_loading();

    view! {
        <div class="field">
            <label class="label">Setup-String</label>
                {
                    move || match setup_string.get() {
                        Some(Some(setup_string)) => {
                            view! {
                                <div class="control is-flex is-justify-content-center">
                                    <textarea class="textarea" placeholder="" prop:value=setup_string readonly></textarea>
                                </div>
                            }
                        }
                        _ => {
                            view! {
                                <div class="control is-flex is-flex-direction-column">
                                    <div class="notification is-warning">
                                        <div class="columns is-mobile is-vcentered">
                                            <div class="column is-narrow">
                                                <i class="fa-solid fa-triangle-exclamation fa-2xl"></i>
                                            </div>
                                            <div class="column">
                                                <p>"After generating a new Setup-String, the peer will "<b>"not be usable in clusters"</b>" until you re-run the setup with the newly generated Setup-String!"</p>
                                            </div>
                                        </div>
                                    </div>
                                    <div class="is-flex is-justify-content-center">
                                        <SimpleButton
                                            text="Generate"
                                            color=ButtonColor::Info
                                            state=button_state
                                            on_action=move || trigger_generation.set(Some(peer_configuration.get().id))
                                        />
                                    </div>
                                </div>
                            }
                        }
                    }
                }
                <br/>
                <p>"Setup-Strings may only be used to set up one host. For setting up multiple hosts, you should create a peer for each host."</p>
        </div>
    }
}
