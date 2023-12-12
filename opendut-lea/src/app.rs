use gloo_net::http;
use leptos::*;
use serde::Deserialize;
use url::Url;

use opendut_carl_api::carl::wasm::CarlClient;

use crate::nav::Navbar;
use crate::routing::Routes;

#[derive(Clone, Debug)]
pub struct AppGlobals {
    pub config: AppConfig,
    pub client: CarlClient,
}

pub fn use_app_globals() -> Resource<(), AppGlobals> {
    use_context::<Resource<(), AppGlobals>>()
        .expect("The AppGlobals should be provided in the context.")
}

#[derive(Clone, Debug, Deserialize)]
pub struct AppConfig {
    carl_url: Url
}

#[component]
pub fn App() -> impl IntoView {

    let globals = create_local_resource(|| {}, |_| async move {
        let config = http::Request::get("/api/lea/config")
            .send()
            .await
            .expect("Should be possible to fetch lea's config")
            .json::<AppConfig>()
            .await
            .expect("Should be possible to parse lea's config");

        log::info!("Configuration: {config:?}");

        let client = CarlClient::create(Clone::clone(&config.carl_url))
            .expect("Failed to create CARL client");

        AppGlobals {
            config,
            client
        }
    });

    provide_context(globals);

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

impl ExpectGlobals for Resource<(), AppGlobals> {

    fn expect_config(&self) -> AppConfig {
        self.get().expect("AppGlobals should be loaded to get the config").config
    }

    fn expect_client(&self) -> CarlClient {
        self.get().expect("AppGlobals should be loaded to get the client").client
    }
}
