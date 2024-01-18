use std::rc::Rc;
use gloo_net::http;
use leptos::*;
use serde::Deserialize;
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
}

pub fn use_app_globals() -> Resource<(), Result<AppGlobals, AppGlobalsError>> {
    use_context::<Resource<(), Result<AppGlobals, AppGlobalsError>>>()
        .expect("The AppGlobals should be provided in the context.")
}

#[derive(Clone, Debug, Deserialize)]
pub struct AppConfig {
    pub carl_url: Url
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

        let client = CarlClient::create(Clone::clone(&config.carl_url))
            .expect("Failed to create CARL client");

        Ok(AppGlobals {
            config,
            client
        })
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
}
