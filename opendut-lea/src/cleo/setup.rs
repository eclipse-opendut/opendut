use leptos::{component, create_local_resource, IntoView, MaybeSignal, ReadSignal, RwSignal, SignalGet, SignalSet, use_context, view, WriteSignal};
use opendut_auth::public::OptionalAuthData;
use crate::app::{ExpectGlobals, use_app_globals};
use crate::components::{BasePageContainer, Breadcrumb, ButtonColor, ButtonStateSignalProvider, Initialized, SimpleButton};

#[component(transparent)]
pub fn CleoSetup() -> impl IntoView {

    #[component]
    fn inner() -> impl IntoView {
        let globals = use_app_globals();

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

        let breadcrumbs = {
            MaybeSignal::derive(move || {
                vec![
                    Breadcrumb::new("Dashboard", "/"),
                    Breadcrumb::new("Cleo", "/cleo"),
                ]
            })
        };

        view! {
            <BasePageContainer
                    title="CLEO Setup"
                    breadcrumbs=breadcrumbs
                    controls=view! { }
            >
                <div class="field">
                    <label class="label">Setup-String</label>
                        {
                            move || match setup_string.get() {
                                Some(Some(setup_string)) => {
                                    view! {
                                        <div class="control is-flex is-justify-content-center">
                                            <textarea class="textarea" rows="20" placeholder="" prop:value=setup_string readonly></textarea>
                                        </div>
                                    }
                                }
                                _ => {
                                    view! {
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
            </BasePageContainer>
        }
    }

    view! {
        <Initialized>
            <Inner />
        </Initialized>
    }
}