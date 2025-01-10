use std::sync::Arc;

use gloo_net::http;
use leptos::either::Either;
use leptos::prelude::*;
use leptos_oidc::{Auth, AuthParameters};
use serde::{Deserialize, Deserializer};
use tracing::info;
use url::Url;

use opendut_carl_api::carl::wasm::CarlClient;

use crate::components::Toaster;
use crate::error::LeaError;
use crate::nav::Navbar;
use crate::routing::AppRoutes;

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
    
    provide_context(Arc::new(Toaster::new()));
    
    view! {
        <Navbar />
        <div class="container">
            <AppRoutes />
        </div>
    }
}
