use leptos::{either::Either, prelude::*};
use crate::components::{use_toaster, ButtonColor, ButtonState, SimpleButton, Toast, WarningMessage};
use opendut_types::peer::PeerId;
use crate::app::use_app_globals;
use crate::user::UserAuthenticationSignal;

#[component]
pub fn GenerateSetupStringForm(kind: Signal<GenerateSetupStringKind>) -> impl IntoView {

    let globals = use_app_globals();

    let trigger_setup_generation: RwSignal<bool> = RwSignal::new(false);

    let setup_string = LocalResource::new(move || {
        let mut carl = globals.client.clone();

        async move {
            let user = use_context::<UserAuthenticationSignal>().expect("UserAuthenticationSignal should be provided in the context.");
            let user_id = user.get().username();

            let trigger = trigger_setup_generation.get();
            if trigger {
                let setup_string = match kind.get() {
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
        <WarningMessage>"Please keep Setup-Strings secret and handle them with care like your passwords."</WarningMessage>
    }
}

#[derive(Clone, Copy)]
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
                    <CopyToClipboardButton setup_string=setup_string_for_clipboard_fn />
                </div>
            </div>
            <div class="control is-flex is-justify-content-center">
                <textarea class="textarea" rows="20" placeholder="" prop:value=setup_string readonly></textarea>
            </div>
        </div>
    }
}

#[component]
fn CopyToClipboardButton(
    setup_string: String
) -> impl IntoView {
    let toaster = use_toaster();

    let copy_action = Action::new_local(move |clipboard_text: &String| {
        let text = clipboard_text.clone();
        let toaster = toaster.clone();

        async move {
            let clipboard = window().navigator().clipboard();
            let clipboard_promise = clipboard.write_text(&text);

            match wasm_bindgen_futures::JsFuture::from(clipboard_promise).await {
                Ok(_) => {
                    toaster.toast(
                        Toast::builder()
                            .simple("Successfully copied Setup-String.")
                            .success(),
                    );
                }
                Err(_) => {
                    toaster.toast(
                        Toast::builder()
                            .simple("Error while copying Setup-String.")
                            .error(),
                    );
                }
            };
        }
    });

    view! {
        <button
            class="button is-light"
            title="Copy to clipboard"
            on:click=move |_| {
                copy_action.dispatch(setup_string.clone());
            }
        >
            <span class="icon">
                <i class="fa-regular fa-copy"></i>
            </span>
        </button>
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
