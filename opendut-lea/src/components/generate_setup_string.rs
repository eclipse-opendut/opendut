use leptos::{either::Either, prelude::*};
use opendut_auth::public::OptionalAuthData;
use opendut_types::peer::PeerId;

use crate::{app::use_app_globals, components::{ButtonColor, ButtonState, SimpleButton}, user::UNAUTHENTICATED_USER, util::clipboard};


#[component]
pub fn GenerateSetupStringForm(kind: GenerateSetupStringKind) -> impl IntoView {

    let globals = use_app_globals();

    let trigger_setup_generation: RwSignal<bool> = RwSignal::new(false);

    let (auth_data_signal, _) = use_context::<(ReadSignal<OptionalAuthData>, WriteSignal<OptionalAuthData>)>().expect("AuthData should be provided in the context.");

    let setup_string = LocalResource::new(move || {
        let mut carl = globals.client.clone();
        async move {
            let trigger = trigger_setup_generation.get();

            let user_id = match auth_data_signal.get().auth_data {
                None => {
                    String::from(UNAUTHENTICATED_USER)
                }
                Some(auth_data) => { auth_data.subject }
            };
            if trigger {
                let setup_string = match kind {
                    GenerateSetupStringKind::Edgar(peer_id) => {
                        let setup = carl.peers.create_peer_setup(peer_id, user_id.clone()).await
                            .expect("Failed to request the setup string.");
                        setup.encode()
                            .expect("PeerSetup should be encodable into a setup-string")
                    },
                    GenerateSetupStringKind::Cleo => {
                        let setup = carl.peers.create_cleo_setup(user_id.clone()).await
                            .expect("Failed to request the setup string.");
                        setup.encode()
                            .expect("CleoSetup should be encodable into a setup-string")
                    }
                };

                Some(setup_string)
            } else {
                None
            }
        }
    });

    view! {
        <Suspense fallback=move || view!{ <GenerateSetupStringButtonForm trigger_setup_generation button_state=ButtonState::Loading/> } >
            {move || Suspend::new(async move {
                let setup_string = setup_string.await;

                match setup_string {
                    Some(setup_string) => {
                        Either::Right(view! { <GenerateSetupStringTextForm setup_string/> })
                    }
                    None => {
                        Either::Left(view! { <GenerateSetupStringButtonForm trigger_setup_generation button_state=ButtonState::Enabled/> })
                    }
                }
            })}
        </Suspense>
        <br/>
        <div class="notification is-warning">
            <div class="columns is-mobile is-vcentered">
                <div class="column is-narrow">
                    <i class="fa-solid fa-triangle-exclamation fa-2xl"></i>
                </div>
                <div class="column">
                    <p>"Please keep Setup-Strings secret and handle them with care like your passwords."</p>
                </div>
            </div>
        </div>
    }
}

pub enum GenerateSetupStringKind {
    Edgar(PeerId),
    Cleo,
}

#[component]
fn GenerateSetupStringTextForm(
    setup_string: String,
) -> impl IntoView {
    let setup_string_for_clipboard_fn = setup_string.clone();

    view! {
        <div>
            <div class="columns mb-0 is-align-items-center">
                <div class="column"><label class="label">Setup-String</label></div>
                <div class="column is-narrow">
                    <button
                        class="button is-light"
                        title="Copy to clipboard"
                        on:click=move |_| {
                            clipboard::copy_with_feedback().dispatch(setup_string_for_clipboard_fn.clone());
                        }
                    >
                        <span class="icon">
                            <i class="fa-regular fa-copy"></i>
                        </span>
                    </button>
                </div>
            </div>
            <div class="control is-flex is-justify-content-center">
                <textarea class="textarea" rows="20" placeholder="" prop:value=setup_string readonly></textarea>
            </div>
        </div>
    }
}

#[component]
fn GenerateSetupStringButtonForm(
    trigger_setup_generation: RwSignal<bool>,
    button_state: ButtonState,
) -> impl IntoView {
    view! {
        <div>
            <label class="label">Setup-String</label>
            <div class="control is-flex is-flex-direction-column">
                <div class="is-flex is-justify-content-center">
                    <SimpleButton
                        text="Generate"
                        color=ButtonColor::Info
                        state=button_state
                        on_action=move || trigger_setup_generation.set(true)
                    />
                </div>
            </div>
        </div>
    }
}
