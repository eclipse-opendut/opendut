use std::sync::Arc;
use gloo_net::http;
use leptos::ev;
use leptos::prelude::*;
use leptos_oidc::{Auth, AuthParameters, AuthSignal};
use leptos_use::{use_document, use_event_listener};
use tracing::{info, warn};
use url::Url;
use opendut_auth::public::Authentication;
use opendut_carl_api::carl::wasm::CarlClient;
use opendut_lea_components::has_text_selection;
use opendut_model::lea::LeaConfig;
use crate::components::Toaster;
use crate::components::AppGlobalsResource;
use crate::nav::{Navbar, Sidebar};
use crate::routing::AppRoutes;
use crate::user::{provide_authentication_signals_in_context, AuthenticationConfigSwitch, UserAuthenticationSignal};

#[derive(Clone, Debug)]
pub struct AppGlobals {
    #[allow(unused)]  // TODO: use carl url as base in use_navigate/navigate_to
    pub config: AppConfig,
    pub client: CarlClient,
    pub auth: Authentication,
}

pub fn use_app_globals() -> AppGlobals {
    use_context::<AppGlobals>()
        .expect("The AppGlobals should be provided in the context.")
}

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub carl_url: Url,
    pub auth_parameters: Option<AuthParameters>,
    pub footer: Option<String>,
}


#[derive(thiserror::Error, Clone, Debug)]
#[error("{message}")]
pub struct AppGlobalsError {
    pub message: String
}

#[derive(Clone)]
pub struct SelectionContext {
    pub has_selection: RwSignal<bool>,
}


#[component]
pub fn LoadingApp() -> impl IntoView {
    let _ = provide_authentication_signals_in_context();

    let app_globals: AppGlobalsResource = LocalResource::new(move || {
        async {
            let config = {
                let LeaConfig { carl_url, idp_config, footer_available } = http::Request::get("/api/lea/config")
                    .send().await
                    .map_err(|cause| AppGlobalsError { message: format!("Could not fetch configuration:\n  {cause}")})?
                    .json::<LeaConfig>().await
                    .map_err(|cause| AppGlobalsError { message: format!("Could not parse configuration:\n  {cause}")})?;


                let footer = if footer_available {
                    let footer = http::Request::get("/api/footer.html")
                        .send().await;

                    match footer {
                        Ok(footer) => {
                            footer.text().await
                                .inspect_err(|cause| warn!("Failed to parse footer as text: {cause}"))
                                .ok()
                        }
                        Err(cause) => {
                            warn!("Failed to fetch footer: {cause}");
                            None
                        }
                    }
                } else {
                    None
                };

                let auth_parameters = idp_config.map(|idp_config| {
                    let redirect_uri = carl_url.to_string();
                    let post_logout_redirect_uri = carl_url.to_string();

                    AuthParameters {
                        // Issuer URL is expected to have no trailing slash
                        issuer: idp_config.issuer_url.to_string().trim_end_matches('/').to_string(),
                        client_id: idp_config.client_id,
                        redirect_uri,
                        post_logout_redirect_uri,
                        challenge: Default::default(),
                        scope: Some(idp_config.scopes),
                        audience: None,
                    }
                });

                AppConfig {
                    carl_url,
                    auth_parameters,
                    footer,
                }
            };

            info!("Configuration: {config:?}");

            let maybe_auth = match config.auth_parameters {
                Some(ref auth_parameters) => {
                    info!("Auth parameters: {auth_parameters:?}");
                    let _ = Auth::init(auth_parameters.clone());
                    let auth = use_context::<AuthSignal>().expect("AuthSignal should be provided in app_globals.");
                    Authentication::Enabled(auth)
                },
                None => Authentication::Disabled
            };
            
            let auth_config_switch = use_context::<RwSignal<AuthenticationConfigSwitch>>().expect("RwSignal<AuthenticationConfigSwitch> should be provided in the context.");
            match maybe_auth {
                Authentication::Disabled => {
                    auth_config_switch.set(AuthenticationConfigSwitch::Disabled);
                }
                Authentication::Enabled(_) => {
                    auth_config_switch.set(AuthenticationConfigSwitch::Enabled);
                }
            }

            let client = CarlClient::create(Clone::clone(&config.carl_url), maybe_auth.clone()).await
                .expect("Failed to create CARL client");

            Ok(AppGlobals {
                config,
                client,
                auth: maybe_auth,
            })
        }
    });

    provide_context(Arc::new(Toaster::new()));

    let menu_visible = RwSignal::new(false);

    let user = use_context::<UserAuthenticationSignal>().expect("UserAuthenticationSignal should be provided in the context.");
    let hide_buttons = Signal::derive(move || !user.read().is_authenticated().unwrap_or(false));

    let has_selection = RwSignal::new(false);
    provide_context(SelectionContext { has_selection });
    
    let context = use_context::<SelectionContext>()
        .expect("SelectionContext should be provided in the context.");
    let _ = use_event_listener(use_document(), ev::selectionchange, move |_| {
        let new_value = has_text_selection();
        if context.has_selection.get_untracked() != new_value {
            context.has_selection.set(new_value);
        }
    });

    view! {
        <div style="display: flex; flex-direction: column; height: 100vh;"> //allows putting footer at bottom
            <Navbar menu_visible hide_buttons />
            <div class="columns is-mobile m-0">
                <Sidebar menu_visible hide_buttons />
                <main class="container column pt-4">
                    <AppRoutes app_globals />
                </main>
            </div>
            <Footer app_globals />
        </div>
    }
}


#[component]
fn Footer(app_globals: AppGlobalsResource) -> impl IntoView {
    view! {
        <Transition>
            {move || Suspend::new(async move {
                app_globals.await
                    .map(|app_globals| app_globals.config.footer)
                    .ok()
                    .flatten()
                    .map(|footer| view! {
                        <footer
                            class="dut-footer mt-auto p-2 has-text-centered"
                            inner_html=footer
                        />
                    })
            })}
        </Transition>
    }
}
