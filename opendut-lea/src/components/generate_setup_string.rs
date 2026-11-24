use crate::util::client_error_description;
use leptos::{either::Either, prelude::*};
use opendut_lea_components::{ButtonSize, ErrorMessage, ErrorText, FontAwesomeIcon, IconButton};
use crate::components::{use_toaster, ButtonColor, ButtonState, SimpleButton, Toast, WarningMessage};
use opendut_model::peer::PeerId;
use crate::app::use_app_globals;
use crate::user::UserAuthenticationSignal;

#[component]
pub fn GenerateSetupStringForm(kind: Signal<GenerateSetupStringKind>) -> impl IntoView {

    let globals = use_app_globals();

    let trigger_setup_generation: RwSignal<bool> = RwSignal::new(false);

    let setup_string = LocalResource::new(move || {
        let mut carl = globals.client.clone();
        let trigger = trigger_setup_generation.get();
        let kind = kind.get();

        async move {
            let user = use_context::<UserAuthenticationSignal>().expect("UserAuthenticationSignal should be provided in the context.");
            let user_id = user.get().username();

            if trigger {
                let setup_string = match kind {
                    GenerateSetupStringKind::Edgar(peer_id) => {
                        let setup = carl.peers.create_peer_setup(peer_id, user_id.clone()).await
                            .map_err(|error| {
                                let error_description = client_error_description(
                                    &error,
                                    "The Setup-String for that peer could not be generated. Please check your connection and try again.",
                                );
                                ErrorText {
                                    headline: String::from("Could not generate Setup-String"),
                                    description: error_description
                                }
                            })?;
                        setup.encode()
                            .map_err(|error| ErrorText {
                                headline: String::from("Could not encode Setup-String"),
                                description: error.to_string()
                            })?
                    },
                    GenerateSetupStringKind::Cleo => {
                        let setup = carl.peers.create_cleo_setup(user_id.clone()).await
                            .map_err(|error| {
                                let error_description = client_error_description(
                                    &error,
                                    "The Setup-String for CLEO could not be generated. Please check your connection and try again.",
                                );
                                ErrorText {
                                    headline: String::from("Could not generate Setup-String"),
                                    description: error_description
                                }
                            })?;
                        setup.encode()
                            .map_err(|error| ErrorText {
                                headline: String::from("Could not encode Setup-String"),
                                description: error.to_string()
                            })?
                    }
                };

                Ok(Some(setup_string))
            } else {
                Ok(None)
            }
        }
    });

    view! {
        <Suspense fallback=move || view!{ <GenerateSetupStringButtonForm trigger_setup_generation button_state=ButtonState::Loading/> } >
            {move || Suspend::new(async move {
                let setup_string = setup_string.await;

                match setup_string {
                    Ok(Some(setup_string)) => {
                        Either::Right(view! { <GenerateSetupStringTextForm setup_string /> })
                    }
                    result => {
                        Either::Left(view! { <GenerateSetupStringButtonForm trigger_setup_generation button_state=ButtonState::Enabled error=result.err() /> })
                    }
                }
            })}
        </Suspense>
        <br/>
        <WarningMessage
            text="Please keep Setup-Strings secret and handle them with care, like your passwords."
        />
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
    setup_string: String,
) -> impl IntoView {
    let toaster = use_toaster();

    let copy_action = Callback::new(move |clipboard_text: String| {
        let text = clipboard_text.clone();
        let toaster = toaster.clone();

        leptos::task::spawn_local(async move {
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
        });
    });

    let auto_copy = RwSignal::new(true);

    Effect::new({
        let setup_string = Clone::clone(&setup_string);
        move || {
            if auto_copy.get() {
                auto_copy.set(false);
                copy_action.run(setup_string.clone());
            }
        }
    });

    view! {
        <IconButton
            icon=FontAwesomeIcon::Copy
            color=ButtonColor::Light
            size=ButtonSize::Normal
            state=ButtonState::Enabled
            label="Copy to clipboard"
            on_action=move || {
                copy_action.run(setup_string.clone());
            }
        />
    }
}

#[component]
fn GenerateSetupStringButtonForm(
    trigger_setup_generation: RwSignal<bool>,
    button_state: ButtonState,
    #[prop(optional_no_strip)] error: Option<ErrorText>,
) -> impl IntoView {
    let has_error = error.is_some();
    view! {
        <div>
            <label class="label">Setup-String</label>
            {error.map(|error| view! { <ErrorMessage text=error /> })}
            <Show when=move || !has_error>
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
            </Show>
        </div>
    }
}
