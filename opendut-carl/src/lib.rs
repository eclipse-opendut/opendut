extern crate core;

use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;

use anyhow::{anyhow, Context, Result};
use axum::extract::{FromRef, State};
use axum::Json;
use axum::routing::get;
use axum_server::tls_rustls::RustlsConfig;
use axum_server_dual_protocol::ServerExt;
use config::Config;
use futures::future::BoxFuture;
use futures::TryFutureExt;
use http::{header::CONTENT_TYPE, Request};
use pem::Pem;
use serde::Serialize;
use tokio::fs;
use tonic::transport::Server;
use tower::{BoxError, make::Shared, ServiceExt, steer::Steer};
use tower_http::services::{ServeDir, ServeFile};
use tracing::{debug, info};
use url::Url;
use shadow_rs::formatcp;
use opendut_carl_api::carl::auth::auth_config::OidcIdentityProviderConfig;
use itertools::Itertools;

use opendut_util::{logging, project};
use opendut_util::logging::LoggingConfig;
use opendut_util::settings::LoadedConfig;

use crate::cluster::manager::{ClusterManager, ClusterManagerOptions, ClusterManagerRef};
use crate::grpc::{ClusterManagerService, MetadataProviderService, PeerManagerService, PeerMessagingBrokerService};
use crate::peer::broker::{PeerMessagingBroker, PeerMessagingBrokerOptions, PeerMessagingBrokerRef};
use crate::peer::oidc_client_manager::{CarlIdentityProviderConfig, OpenIdConnectClientManager};
use crate::resources::manager::{ResourcesManager, ResourcesManagerRef};
use crate::vpn::Vpn;

pub mod grpc;

opendut_util::app_info!();

mod actions;
mod cluster;
mod peer;
mod resources;
pub mod settings;
mod vpn;

#[tracing::instrument]
pub async fn create_with_logging(settings_override: config::Config) -> Result<()> {
    let settings = settings::load_with_overrides(settings_override)?;

    let logging_config = LoggingConfig::load(&settings.config)?;
    let mut shutdown = logging::initialize_with_config(logging_config)?;

    create(settings).await?;

    shutdown.shutdown();

    Ok(())
}

pub async fn create(settings: LoadedConfig) -> Result<()> { //TODO
    info!("Started with configuration: {settings:?}");

    let address: SocketAddr = {
        let host = settings.config.get_string("network.bind.host")?;
        let port = settings.config.get_int("network.bind.port")?;
        SocketAddr::from_str(&format!("{host}:{port}"))?
    };

    let tls_config = {
        let cert_path = project::make_path_absolute(settings.config.get_string("network.tls.certificate")?)?;
        debug!("Using TLS certificate: {}", cert_path.display());
        assert!(cert_path.exists(), "TLS certificate file at '{}' not found.", cert_path.display());

        let key_path = project::make_path_absolute(settings.config.get_string("network.tls.key")?)?;
        debug!("Using TLS key: {}", key_path.display());
        assert!(key_path.exists(), "TLS key file at '{}' not found.", key_path.display());

        RustlsConfig::from_pem_file(cert_path, key_path).await?
    };


    let ca_string = fs::read_to_string(project::make_path_absolute(settings.config.get_string("network.tls.ca")?)?).await?;
    let ca_certificate = match Pem::from_str(ca_string.as_str()) {
        Ok(pem) => { pem }
        Err(error) => {
            panic!("Missing CA certificate file in CARL's configuration TOML. {:?}", error)
        }
    };


    let vpn = vpn::create(&settings.config)
        .context("Error while parsing VPN configuration.")?;

    let carl_url = {
        let host = settings.config.get_string("network.remote.host").expect("Configuration value for 'network.remote.host' should be set.");
        let port = settings.config.get_int("network.remote.port").expect("Configuration value for 'network.remote.port' should be set.");
        Url::parse(&format!("https://{host}:{port}"))
            .context(format!("Could not create CARL URL from given host '{host}' and {port}."))?
    };


    let resources_manager = ResourcesManager::new();
    let peer_messaging_broker = PeerMessagingBroker::new(
        Arc::clone(&resources_manager),
        PeerMessagingBrokerOptions::load(&settings.config)?,
    );
    let cluster_manager = ClusterManager::new(
        Arc::clone(&resources_manager),
        Arc::clone(&peer_messaging_broker),
        Clone::clone(&vpn),
        ClusterManagerOptions::load(&settings.config)?,
    );

    /// Isolation in function returning BoxFuture needed due to this: https://github.com/rust-lang/rust/issues/102211#issuecomment-1397600424
    #[allow(clippy::too_many_arguments)]
    #[tracing::instrument(skip(peer_messaging_broker, cluster_manager, resources_manager, vpn), level="TRACE")]
    fn spawn_server(
        address: SocketAddr,
        tls_config: RustlsConfig,
        resources_manager: ResourcesManagerRef,
        cluster_manager: ClusterManagerRef,
        peer_messaging_broker: PeerMessagingBrokerRef,
        vpn: Vpn,
        carl_url: Url,
        settings: config::Config,
        ca: Pem,
    ) -> BoxFuture<'static, Result<()>> {
        let oidc_enabled = settings.get_bool("network.oidc.enabled").unwrap_or(false);

        let oidc_client_manager = if oidc_enabled {
            let carl_oidc_config = CarlIdentityProviderConfig::try_from(&settings)
                .expect("Failed to create CarlIdentityProviderConfig from settings.");
            Some(OpenIdConnectClientManager::new(carl_oidc_config)
                .expect("Failed to create OpenIdConnectClientManager."))
        } else {
            None
        };

        let grpc = Server::builder()
            .accept_http1(true) //gRPC-web uses HTTP1
            .add_service(
                ClusterManagerService::new(Arc::clone(&cluster_manager), Arc::clone(&resources_manager))
                    .into_grpc_service()
            )
            .add_service(
                MetadataProviderService::new()
                    .into_grpc_service()
            )
            .add_service(
                PeerManagerService::new(Arc::clone(&resources_manager), vpn, Clone::clone(&carl_url), ca, oidc_client_manager)
                    .into_grpc_service()
            )
            .add_service(
                PeerMessagingBrokerService::new(Arc::clone(&peer_messaging_broker))
                    .into_grpc_service()
            )
            .into_service()
            .map_response(|response| response.map(axum::body::boxed))
            .boxed_clone();

        let lea_dir = project::make_path_absolute(settings.get_string("serve.ui.directory")
            .expect("Failed to find configuration for `serve.ui.directory`."))
            .expect("Failure while making path absolute.");
        let lea_presence_check = settings.get_bool("serve.ui.presence_check").unwrap_or(true);
        let licenses_dir = project::make_path_absolute("./licenses")
            .expect("licenses directory should be absolute");

        let lea_idp_config = if oidc_enabled {
            let lea_idp_config = LeaIdentityProviderConfig::try_from(&settings)
                .expect("Failed to create LeaIdentityProviderConfig from settings.");
            info!("OIDC is enabled.");
            Some(lea_idp_config)
        } else {
            info!("OIDC is disabled.");
            None
        };

        let app_state = AppState {
            lea_config: LeaConfig {
                carl_url,
                idp_config: lea_idp_config,
            }
        };

        let lea_index_html = lea_dir.join("index.html").clone();
        if lea_presence_check {
            // Check if LEA can be served
            if lea_index_html.exists() {
                let lea_index_str = std::fs::read_to_string(lea_index_html.clone()).expect("Failed to read LEA index.html");
                if !lea_index_str.contains("bg.wasm") || !lea_index_str.contains("opendut-lea") {
                    panic!("LEA index.html does not contain wasm link! Check configuration serve.ui.directory={:?} points to the correct directory.", lea_dir.into_os_string());
                }
            } else {
                panic!("Failed to check if LEA index.html exists in: {}", lea_index_html.display());
            }
        }
        let http = axum::Router::new()
            .fallback_service(
                axum::Router::new()
                    .nest_service(
                        "/api/licenses",
                        ServeDir::new(&licenses_dir)
                            .fallback(ServeFile::new(licenses_dir.join("index.json")))
                    )
                    .route("/api/lea/config", get(lea_config))
                    .nest_service(
                        "/",
                        ServeDir::new(&lea_dir)
                            .fallback(ServeFile::new(lea_index_html))
                    )
                    .with_state(app_state)
            )
            .map_err(BoxError::from)
            .boxed_clone();

        let http_grpc = Steer::new(vec![grpc, http], |request: &Request<_>, _services: &[_]| {
            request.headers()
                .get(CONTENT_TYPE)
                .map(|content_type| {
                    let content_type = content_type.as_bytes();

                    if content_type.starts_with(b"application/grpc") {
                        0
                    } else {
                        1
                    }
                }).unwrap_or(1)
        });

        Box::pin(
            axum_server_dual_protocol::bind_dual_protocol(address, tls_config)
                .set_upgrade(true) //http -> https
                .serve(Shared::new(http_grpc))
                .map_err(|cause| anyhow!(cause))
        )
    }

    info!("Server listening at {address}...");
    spawn_server(
        address,
        tls_config,
        resources_manager,
        cluster_manager,
        peer_messaging_broker,
        vpn,
        carl_url,
        settings.config,
        ca_certificate,
    ).await.unwrap();

    Ok(())
}

#[derive(Clone)]
struct AppState {
    lea_config: LeaConfig
}

#[derive(Clone, Debug, Serialize)]
struct LeaIdentityProviderConfig {
    client_id: String,
    issuer_url: Url,
    scopes: String,
}

const LEA_OIDC_CONFIG_PREFIX: &str = "network.oidc.lea";
impl TryFrom<&Config> for LeaIdentityProviderConfig {
    type Error = anyhow::Error;

    fn try_from(config: &Config) -> Result<Self> {

        let client_id = config.get_string(LeaIdentityProviderConfig::CLIENT_ID)
            .map_err(|error| anyhow!("Failed to find configuration for `{}`. {}", LeaIdentityProviderConfig::CLIENT_ID, error))?;
        let issuer = config.get_string(LeaIdentityProviderConfig::ISSUER_URL)
            .map_err(|error| anyhow!("Failed to find configuration for `{}`. {}", LeaIdentityProviderConfig::ISSUER_URL, error))?;

        let issuer_url = Url::parse(&issuer)
            .context(format!("Failed to parse OIDC issuer URL `{}`.", issuer))?;

        let lea_raw_scopes = config.get_string(LeaIdentityProviderConfig::SCOPES)
            .map_err(|error| anyhow!("Failed to find configuration for `{}`. {}", LeaIdentityProviderConfig::SCOPES, error))?;

        let scopes = OidcIdentityProviderConfig::parse_scopes(&client_id, lea_raw_scopes).into_iter()
            .map(|scope| scope.to_string()).join(" ");  // Required by leptos_oidc

        Ok(Self { client_id, issuer_url, scopes })
    }
}
impl LeaIdentityProviderConfig {
    const CLIENT_ID: &'static str = formatcp!("{LEA_OIDC_CONFIG_PREFIX}.client.id");
    const ISSUER_URL: &'static str = formatcp!("{LEA_OIDC_CONFIG_PREFIX}.issuer.url");
    const SCOPES: &'static str = formatcp!("{LEA_OIDC_CONFIG_PREFIX}.scopes");
}

#[derive(Clone, Serialize)]
struct LeaConfig {
    carl_url: Url,
    idp_config: Option<LeaIdentityProviderConfig>,
}

impl FromRef<AppState> for LeaConfig {
    fn from_ref(app_state: &AppState) -> Self {
        Clone::clone(&app_state.lea_config)
    }
}

async fn lea_config(State(config): State<LeaConfig>) -> Json<LeaConfig> {
    Json(Clone::clone(&config))
}
