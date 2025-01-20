
use std::sync::Arc;

use gloo_net::http;
use leptos::prelude::*;
use leptos_oidc::{Auth, AuthParameters};
use leptos_router::hooks::use_navigate;
use serde::{Deserialize, Deserializer};
use tracing::{error, info};
use url::Url;

use opendut_carl_api::carl::wasm::CarlClient;

use crate::components::{AppGlobalsResource, Toaster};
use crate::nav::Navbar;
use crate::routing::{navigate_to, AppRoutes, WellKnownRoutes};

#[derive(Clone, Debug)]
pub struct AppGlobals {
    pub config: AppConfig,
    pub client: CarlClient,
    pub auth: Option<Auth>,
}

pub fn use_app_globals() -> AppGlobals {
    use_context::<AppGlobals>()
        .expect("The AppGlobals should be provided in the context.")
}

#[derive(Clone, Debug, Deserialize)]
pub struct LeaIdpConfig {
    pub client_id: String,
    pub issuer_url: Url,
    pub scopes: String,
}


// TODO: RawAppConfig==LeaConfig(opendut-carl/src/http/state.rs), move to opendut-types
#[derive(Clone, Debug, Deserialize)]
pub struct RawAppConfig {
    pub carl_url: Url,
    pub idp_config: Option<LeaIdpConfig>,
}

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub carl_url: Url,
    pub idp_config: Option<LeaIdpConfig>,
    pub auth_parameters: Option<AuthParameters>,
}

impl<'de> Deserialize<'de> for AppConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        let raw_app_config: RawAppConfig = Deserialize::deserialize(deserializer)?;

        match raw_app_config.idp_config {
            Some(idp_config) => {
                let redirect_uri = raw_app_config.carl_url.to_string();
                let post_logout_redirect_uri = raw_app_config.carl_url.to_string();

                Ok(AppConfig {
                    carl_url: raw_app_config.carl_url,
                    idp_config: Some(idp_config.clone()),
                    auth_parameters: Some(AuthParameters {
                        // Issuer URL is expected to have no trailing slash
                        issuer: idp_config.issuer_url.to_string().trim_end_matches('/').to_string(),
                        client_id: idp_config.client_id,
                        redirect_uri,
                        post_logout_redirect_uri,
                        challenge: Default::default(),
                        scope: Some(idp_config.scopes),
                        audience: None,
                    }),
                })
            },
            None => Ok(AppConfig {
                carl_url: raw_app_config.carl_url,
                idp_config: None,
                auth_parameters: None,
            })
        }
    }
}

#[derive(thiserror::Error, Clone, Debug)]
#[error("{message}")]
pub struct AppGlobalsError {
    pub message: String
}

#[component]
pub fn LoadingApp() -> impl IntoView {
    use leptos_router::components::Router;

    let app_globals: AppGlobalsResource = LocalResource::new(move || {
        async {
            let config = http::Request::get("/api/lea/config")
                .send().await
                .map_err(|cause| AppGlobalsError { message: format!("Could not fetch configuration:\n  {cause}")})?
                .json::<AppConfig>().await
                .map_err(|cause| AppGlobalsError { message: format!("Could not parse configuration:\n  {cause}")})?;

            info!("Configuration: {config:?}");

            let maybe_auth = match config.auth_parameters {
                Some(ref auth_parameters) => {
                    info!("Auth parameters: {auth_parameters:?}");

                    match Auth::init(auth_parameters.clone()).await {
                        Ok(auth) => Some(auth),
                        Err(error) => {
                            let error_message = format!("Error while initializing the authentication stack: {error}");
                            error!(error_message);

                            navigate_to(
                                WellKnownRoutes::ErrorPage {
                                    title: String::from("Initialization error"),
                                    text: error_message,
                                    details: None,
                                },
                                use_navigate()
                            );
                            None
                        }
                    }
                },
                None => None
            };

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

    view! {
        <Router>
            <Navbar app_globals />
            <main class="container">
                <AppRoutes app_globals />
            </main>
        </Router>
    }
}
