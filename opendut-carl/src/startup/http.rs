use anyhow::anyhow;
use axum::routing::get;
use config::Config;
use tower_http::services::{ServeDir, ServeFile};
use tracing::info;
use opendut_auth::registration::resources::ResourceHomeUrl;
use opendut_model::lea::LeaConfig;
use opendut_util::project;
use crate::http::router;
use crate::http::state::{CarlInstallDirectory, HttpState, LoadableLeaIdentityProviderConfig};


pub fn create_http_service(settings: &Config) -> anyhow::Result<axum::Router<HttpState>> {

    let lea_dir = project::make_path_absolute(settings.get_string("serve.ui.directory")
        .expect("Failed to find configuration for `serve.ui.directory`."))
        .expect("Failure while making path absolute.");
    let lea_index_html = lea_dir.join("index.html");

    let lea_presence_check = settings.get_bool("serve.ui.presence_check").unwrap_or(true);
    if lea_presence_check {
        // Check if LEA can be served
        if lea_index_html.exists() {
            let lea_index_str = std::fs::read_to_string(&lea_index_html).expect("Failed to read LEA index.html");
            assert!(
                !(!lea_index_str.contains("bg.wasm") || !lea_index_str.contains("opendut-lea")), 
                "LEA index.html does not contain wasm link! Check configuration serve.ui.directory={:?} points to the correct directory.", lea_dir.into_os_string()
            );
        } else {
            panic!("Failed to check if LEA index.html exists in: {}", lea_index_html.display());
        }
    }

    let licenses_dir = project::make_path_absolute("./licenses")
        .expect("licenses directory should be absolute");

    let router = axum::Router::new()
        .nest_service(
            "/api/licenses",
            ServeDir::new(&licenses_dir)
                .fallback(ServeFile::new(licenses_dir.join("index.json")))
        )
        .route("/api/cleo/{architecture}/download", get(router::cleo::download_cleo))
        .route("/api/edgar/{architecture}/download", get(router::edgar::download_edgar))
        .route("/api/lea/config", get(router::lea_config))
        .fallback_service(
            ServeDir::new(&lea_dir)
                .fallback(ServeFile::new(lea_index_html))
        );

    Ok(router)
}

pub fn create_http_state(
    carl_url: &ResourceHomeUrl,
    carl_installation_directory: CarlInstallDirectory,
    settings: &Config,
) -> anyhow::Result<HttpState> {

    let oidc_enabled = settings.get_bool("network.oidc.enabled").unwrap_or(false);
    let lea_idp_config = if oidc_enabled {
        let lea_idp_config = LoadableLeaIdentityProviderConfig::try_from(settings)
            .map_err(|_| anyhow!("Failed to create LeaIdentityProviderConfig from settings."))?;
        info!("OIDC is enabled.");
        Some(lea_idp_config)
    } else {
        info!("OIDC is disabled.");
        None
    };

    let http_state = HttpState {
        lea_config: LeaConfig {
            carl_url: carl_url.value(),
            idp_config: lea_idp_config.map(|LoadableLeaIdentityProviderConfig(config)| config),
        },
        carl_installation_directory
    };

    Ok(http_state)
}
