
use std::sync::Arc;

use gloo_net::http;
use leptos::prelude::*;
use leptos_oidc::{Auth, AuthParameters, AuthSignal};
use serde::{Deserialize, Deserializer};
use tracing::{error, info};
use url::Url;
use opendut_auth::public::Authentication;
use opendut_carl_api::carl::wasm::CarlClient;
use opendut_types::lea::LeaConfig;
use crate::components::{AppGlobalsResource, Toaster};
use crate::nav::Navbar;
use crate::routing::AppRoutes;
use crate::user::{provide_authentication_signals_in_context, AuthenticationConfigSwitch};

#[derive(Clone, Debug)]
pub struct AppGlobals {
    #[allow(dead_code)]
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
}

impl<'de> Deserialize<'de> for AppConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        let lea_config: LeaConfig = Deserialize::deserialize(deserializer)?;

        match lea_config.idp_config {
            Some(idp_config) => {
                let redirect_uri = lea_config.carl_url.to_string();
                let post_logout_redirect_uri = lea_config.carl_url.to_string();

                Ok(AppConfig {
                    carl_url: lea_config.carl_url,
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
                carl_url: lea_config.carl_url,
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
    let _ = provide_authentication_signals_in_context();

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

    view! {
        <Navbar app_globals />
        <main class="container">
            <AppRoutes app_globals />
        </main>
    }
}
