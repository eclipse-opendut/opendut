use leptos::*;
use crate::app::{use_app_globals, ExpectGlobals};
use opendut_auth::public::OptionalAuthData;
use opendut_types::proto::util::VersionInfo;
use crate::components::{ButtonColor, ButtonStateSignalProvider, SimpleButton};

#[component]
pub fn CleoCard(
    version_info: Resource<(), VersionInfo>
) -> impl IntoView {

    let globals = use_app_globals();

    let version_name = move || {
        match version_info.get() {
            None => { String::new() }
            Some(version) => {
                format!("-{}", version.name)
            }
        }
    };

    let output_name_aarch64 = move || {
        format!("opendut-cleo-aarch64-unknown-linux-gnu{}.tar.gz", version_name())
    };
    let output_name_armv7 = move || {
        format!("opendut-cleo-armv7-unknown-linux-gnueabihf{}.tar.gz", version_name())
    };
    let output_name_x86 = move || {
        format!("opendut-cleo-x86_64-unknown-linux-gnu{}.tar.gz", version_name())
    };

    let trigger_cleo_setup_generation: RwSignal<bool> = RwSignal::new(false);

    let (auth_data_signal, _) = use_context::<(ReadSignal<OptionalAuthData>, WriteSignal<OptionalAuthData>)>().expect("AuthData should be provided in the context.");

    let setup_string = create_local_resource(move || trigger_cleo_setup_generation.get(), move |action| {
        async move {
            let user_id = match auth_data_signal.get().auth_data {
                None => { String::from("UNKNOWN USER") }
                Some(auth_data) => { auth_data.subject }
            };
            if action {
                let mut carl = globals.expect_client();
                let setup = carl.peers.create_cleo_setup(user_id.clone()).await
                    .expect("Failed to request the setup string.");
                let setup_string = setup.encode()
                    .expect("PeerSetup should be encodable into a setup-string");
                Some(setup_string)
            } else {
                None
            }
        }
    });

    let button_state = setup_string.loading().derive_loading();

    view! {
        <div class="card">
            <div class="card-header">
                <div class="card-header-title"><i class="fa-solid fa-terminal mr-1"></i>"CLEO"</div>
            </div>
            <div class="card-content">
            "Download for different architectures:"
                <div class="mb-2 mt-2 ml-2">
                    <a href="/api/cleo/aarch64-unknown-linux-gnu/download" download=output_name_aarch64>
                        <i class="fa-solid fa-download fa-lg pr-1" />
                        <span class="ml-2 is-size-6">"aarch64-gnu"</span>
                    </a>
                </div>
                <div class="mb-2 ml-2">
                    <a href="/api/cleo/armv7-unknown-linux-gnueabihf/download" download=output_name_armv7>
                        <i class="fa-solid fa-download fa-lg pr-1" />
                        <span class="ml-2 is-size-6">"armv7-gnueabihf"</span>
                    </a>
                </div>
                <div class="mb-2 ml-2">
                    <a href="/api/cleo/x86_64-unknown-linux-gnu/download" download=output_name_x86>
                        <i class="fa-solid fa-download fa-lg pr-1" />
                        <span class="ml-2 is-size-6">"x86_64-gnu"</span>
                    </a>
                </div>
                <div class="mt-5">
                    <div class="field">
                        {
                            move || match setup_string.get() {
                                Some(Some(setup_string)) => {
                                    let clipboard_text = setup_string.clone();
                                    view! {
                                        <div>
                                            <div class="columns mb-0 is-align-items-center">
                                                <div class="column"><label class="label">Setup-String</label></div>
                                                <div class="column is-narrow">
                                                    <button
                                                        class="button is-light"
                                                        title="Copy to clipboard"
                                                        on:click=move |_| {
                                                            let clipboard = window().navigator().clipboard();
                                                            let _ = clipboard.write_text(&clipboard_text);
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
                                _ => {
                                    view! {
                                        <div>
                                            <label class="label">Setup-String</label>
                                            <div class="control is-flex is-flex-direction-column">
                                                <div class="is-flex is-justify-content-center">
                                                    <SimpleButton
                                                        text="Generate"
                                                        color=ButtonColor::Info
                                                        state=button_state
                                                        on_action=move || trigger_cleo_setup_generation.set(true)
                                                    />
                                                </div>
                                            </div>
                                        </div>
                                    }
                                }
                            }
                        }
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
                    </div>
                </div>
            </div>
        </div>
    }
}
