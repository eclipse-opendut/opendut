use std::rc::Rc;

use gloo_net::http;
use leptos::*;
use leptos_oidc::{Auth, AuthParameters};
use serde::{Deserialize, Deserializer};
use serde::de::Error;
use tracing::info;
use url::Url;

use opendut_carl_api::carl::wasm::CarlClient;

use crate::components::Toaster;
use crate::nav::Navbar;
use crate::routing::Routes;

#[derive(Clone, Debug)]
pub struct AppGlobals {
    pub config: AppConfig,
    pub client: CarlClient,
    pub auth: Option<Auth>,
}

pub fn use_app_globals() -> Resource<(), Result<AppGlobals, AppGlobalsError>> {
    use_context::<Resource<(), Result<AppGlobals, AppGlobalsError>>>()
        .expect("The AppGlobals should be provided in the context.")
}

#[derive(Clone, Debug, Deserialize)]
pub struct LeaIdpConfig {
    pub client_id: String,
    pub issuer_url: Url,
    pub scopes: String,
}

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
                let auth_endpoint = idp_config.issuer_url.join("protocol/openid-connect/auth")
                    .map_err(Error::custom)?.to_string();
                let token_endpoint = idp_config.issuer_url.join("protocol/openid-connect/token")
                    .map_err(Error::custom)?.to_string();
                let logout_endpoint = idp_config.issuer_url.join("protocol/openid-connect/logout")
                    .map_err(Error::custom)?.to_string();
                let redirect_uri = raw_app_config.carl_url.to_string();
                let post_logout_redirect_uri = raw_app_config.carl_url.to_string();

                Ok(AppConfig {
                    carl_url: raw_app_config.carl_url,
                    idp_config: Some(idp_config.clone()),
                    auth_parameters: Some(AuthParameters {
                        auth_endpoint,
                        token_endpoint,
                        logout_endpoint,
                        client_id: idp_config.client_id,
                        redirect_uri,
                        post_logout_redirect_uri,
                        scope: Some(idp_config.scopes),
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
pub fn App() -> impl IntoView {

    let globals: Resource<(), Result<AppGlobals, AppGlobalsError>> = create_local_resource(|| {}, |_| async move {
        let config = http::Request::get("/api/lea/config")
            .send()
            .await
            .map_err(|_| AppGlobalsError { message: String::from("Could not fetch configuration!")})?
            .json::<AppConfig>()
            .await.map_err(|_| AppGlobalsError { message: String::from("Could not parse configuration!")})?;

        info!("Configuration: {config:?}");

        match config.auth_parameters {
            Some(ref auth_parameters) => {
                info!("Auth parameters: {auth_parameters:?}");
                let auth = Auth::init(auth_parameters.clone());
                let client = CarlClient::create(Clone::clone(&config.carl_url), Some(auth.clone()))
                    .expect("Failed to create CARL client");

                Ok(AppGlobals {
                    config,
                    client,
                    auth: Some(auth),
                })
            },
            None => {
                let client = CarlClient::create(Clone::clone(&config.carl_url), None)
                    .expect("Failed to create CARL client");
                Ok(AppGlobals {
                    config,
                    client,
                    auth: None,
                })
            }
        }
    });

    provide_context(globals);
    provide_context(Rc::new(Toaster::new()));

    view! {
        <Navbar />
        <div class="container">
            <Routes />
        </div>
    }
}

pub trait ExpectGlobals {
    fn expect_config(&self) -> AppConfig;
    fn expect_client(&self) -> CarlClient;
    fn expect_auth(&self) -> Option<Auth>;
}

impl ExpectGlobals for Resource<(), Result<AppGlobals, AppGlobalsError>> {

    fn expect_config(&self) -> AppConfig {
        self.get()
            .expect("AppGlobals should be loaded to get the config")
            .expect("AppGlobals should be loaded successfully to get the config")
            .config
    }

    fn expect_client(&self) -> CarlClient {
        self.get()
            .expect("AppGlobals should be loaded to get the client")
            .expect("AppGlobals should be loaded successfully to get the client")
            .client
    }

    fn expect_auth(&self) -> Option<Auth> {
        self.get()
            .expect("AppGlobals should be loaded to get the authentication")
            .expect("AppGlobals should be loaded successfully to get the authentication")
            .auth
    }
}
