use std::net::SocketAddr;
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
use hyper::Body;
use serde::Serialize;
use tonic::transport::Server;
use tower::{BoxError, make::Shared, ServiceExt, steer::Steer};
use tower_http::services::{ServeDir, ServeFile};
use url::Url;

use opendut_util::{project, settings};

use crate::cluster::manager::{ClusterManager, ClusterManagerRef};
use crate::grpc::{ClusterManagerService, MetadataProviderService, PeerManagerService, PeerMessagingBrokerService};
use crate::peer::broker::broker::{PeerMessagingBroker, PeerMessagingBrokerRef};
use crate::resources::manager::{ResourcesManager, ResourcesManagerRef};
use crate::vpn::Vpn;

pub mod grpc;

opendut_util::app_info!();

mod actions;
mod cluster;
mod peer;
mod resources;
mod vpn;

pub async fn create(settings_override: Config) -> Result<()> {

    let settings = settings::load_config("carl", include_str!("../carl.toml"), config::FileFormat::Toml, settings_override)?;

    log::info!("Started with configuration: {settings:?}");

    let address: SocketAddr = {
        let host = settings.config.get_string("network.bind.host")?;
        let port = settings.config.get_int("network.bind.port")?;
        format!("{host}:{port}").parse()?
    };

    let tls_config = {
        let cert_path = project::make_path_absolute(settings.config.get_string("network.tls.certificate")?)?;
        log::debug!("Using TLS certificate: {}", cert_path.display());
        let key_path = project::make_path_absolute(settings.config.get_string("network.tls.key")?)?;
        log::debug!("Using TLS key: {}", key_path.display());

        RustlsConfig::from_pem_file(cert_path, key_path).await?
    };

    let vpn = vpn::create(&settings.config)
        .context("Error while parsing VPN configuration.")?;

    let carl_url = {
        let host = settings.config.get_string("network.remote.host").expect("Configuration value for 'network.remote.host' should be set.");
        let port = settings.config.get_int("network.remote.port").expect("Configuration value for 'network.remote.port' should be set.");
        Url::parse(&format!("https://{host}:{port}"))
            .context(format!("Could not create CARL URL from given host '{host}' and {port}."))?
    };

    let resources_manager = Arc::new(ResourcesManager::new());
    let peer_messaging_broker = Arc::new(PeerMessagingBroker::new());
    let cluster_manager = Arc::new(ClusterManager::new(
        Arc::clone(&resources_manager),
        Arc::clone(&peer_messaging_broker),
        Clone::clone(&vpn),
    ));

    /// Isolation in function returning BoxFuture needed due to this: https://github.com/rust-lang/rust/issues/102211#issuecomment-1397600424
    fn spawn_server(
        address: SocketAddr,
        tls_config: RustlsConfig,
        resources_manager: ResourcesManagerRef,
        cluster_manager: ClusterManagerRef,
        peer_messaging_broker: PeerMessagingBrokerRef,
        vpn: Vpn,
        carl_url: Url,
        settings: Config
    ) -> BoxFuture<'static, Result<()>> {

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
                PeerManagerService::new(Arc::clone(&resources_manager), vpn, Clone::clone(&carl_url))
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

        let licenses_dir = project::make_path_absolute("./licenses")
            .expect("licenses directory should be absolute");

        let app_state = AppState {
            lea_config: LeaConfig {
                carl_url
            }
        };

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
                            .fallback(ServeFile::new(lea_dir.join("index.html")))
                    )
                    .with_state(app_state)
            )
            .map_err(BoxError::from)
            .boxed_clone();

        let http_grpc = Steer::new(vec![grpc, http], |request: &Request<Body>, _services: &[_]| {
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

    log::info!("Server listening at {address}...");
    spawn_server(
        address,
        tls_config,
        resources_manager,
        cluster_manager,
        peer_messaging_broker,
        vpn,
        carl_url,
        settings.config,
    ).await.unwrap();

    Ok(())
}

#[derive(Clone)]
struct AppState {
    lea_config: LeaConfig
}

#[derive(Clone, Serialize)]
struct LeaConfig {
    carl_url: Url
}

impl FromRef<AppState> for LeaConfig {
    fn from_ref(app_state: &AppState) -> Self {
        Clone::clone(&app_state.lea_config)
    }
}

async fn lea_config(State(config): State<LeaConfig>) -> Json<LeaConfig> {
    Json(Clone::clone(&config))
}
