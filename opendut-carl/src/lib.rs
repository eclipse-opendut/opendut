extern crate core;

use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;

use ::http::{header::CONTENT_TYPE, Request};
use anyhow::{anyhow, Context, Result};
use axum::routing::get;
use axum_server::tls_rustls::RustlsConfig;
use axum_server_dual_protocol::ServerExt;
use futures::future::BoxFuture;
use futures::TryFutureExt;
use pem::Pem;
use tonic::transport::Server;
use tonic_async_interceptor::async_interceptor;
use tower::{BoxError, make::Shared, ServiceExt, steer::Steer};
use tower_http::services::{ServeDir, ServeFile};
use tracing::{debug, info, warn};
use uuid::Uuid;

use opendut_auth::confidential::pem::PemFromConfig;
use opendut_auth::registration::client::{RegistrationClient, RegistrationClientRef};
use opendut_auth::registration::resources::ResourceHomeUrl;
use opendut_util::{project, telemetry};
use opendut_util::settings::LoadedConfig;
use opendut_util::telemetry::logging::LoggingConfig;
use opendut_util::telemetry::opentelemetry_types::Opentelemetry;
use util::in_memory_cache::CustomInMemoryCache;

use crate::auth::grpc_auth_layer::GrpcAuthenticationLayer;
use crate::auth::json_web_key::JwkCacheValue;
use crate::cluster::manager::{ClusterManager, ClusterManagerOptions, ClusterManagerRef};
use crate::grpc::{ClusterManagerFacade, MetadataProviderFacade, PeerManagerFacade, PeerManagerFacadeOptions, PeerMessagingBrokerFacade};
use crate::http::router;
use crate::http::state::{CarlInstallDirectory, HttpState, LeaConfig, LeaIdentityProviderConfig};
use crate::peer::broker::{PeerMessagingBroker, PeerMessagingBrokerOptions, PeerMessagingBrokerRef};
use crate::provisioning::cleo_script::CleoScript;
use crate::resources::manager::{ResourcesManager, ResourcesManagerRef};
use crate::resources::storage::PersistenceOptions;
use crate::vpn::Vpn;

pub mod grpc;
pub mod util;
opendut_util::app_info!();

mod actions;
mod cluster;
mod metrics;
pub mod persistence;
mod peer;
mod resources;
pub mod settings;
mod vpn;
mod http;
mod provisioning;
mod auth;

#[tracing::instrument]
pub async fn create_with_telemetry(settings_override: config::Config) -> Result<()> {
    let settings = settings::load_with_overrides(settings_override)?;

    let service_instance_id = format!("carl-{}", Uuid::new_v4());

    let logging_config = LoggingConfig::load(&settings.config)?;
    let opentelemetry = Opentelemetry::load(&settings.config, service_instance_id).await?;

    let mut shutdown = telemetry::initialize_with_config(logging_config, opentelemetry.clone()).await?;
    
    if let (Opentelemetry::Enabled { cpu_collection_interval_ms, .. },
        Some(meter_providers, ..)) = (opentelemetry, &shutdown.meter_providers) {
        telemetry::metrics::initialize_metrics_collection(cpu_collection_interval_ms, meter_providers);
    }

    create(settings).await?;

    shutdown.shutdown();

    Ok(())
}

pub async fn create(settings: LoadedConfig) -> Result<()> {
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
    let carl_url = ResourceHomeUrl::try_from(&settings.config)?;

    let ca_certificate = Pem::from_config_path("network.tls.ca", &settings.config).await?;
    let oidc_registration_client = RegistrationClient::from_settings(&settings.config).await.expect("Failed to load oidc registration client!");

    let vpn = vpn::create(&settings.config)
        .context("Error while parsing VPN configuration.")?;

    let resources_storage_options = PersistenceOptions::load(&settings.config)?;
    let resources_manager = ResourcesManager::create(resources_storage_options).await
        .context("Creating ResourcesManager failed")?;
    metrics::initialize_metrics_collection(Arc::clone(&resources_manager));

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

    let jwk_cache: CustomInMemoryCache<String, JwkCacheValue> = CustomInMemoryCache::new();

    let grpc_auth_layer = match oidc_registration_client.clone() {
        None => {
            GrpcAuthenticationLayer::AuthDisabled
        }
        Some(oidc_client_ref) => {
            GrpcAuthenticationLayer::GrpcAuthLayerEnabled {
                issuer_url: oidc_client_ref.inner.config.issuer_url.clone(),
                issuer_remote_url: oidc_client_ref.config.issuer_remote_url.clone(),
                cache: jwk_cache,
            }
        }
    };

    /// Isolation in function returning BoxFuture needed due to this: https://github.com/rust-lang/rust/issues/102211#issuecomment-1397600424
    #[allow(clippy::too_many_arguments)]
    #[tracing::instrument(skip_all, level="TRACE")]
    fn spawn_server(
        address: SocketAddr,
        tls_config: RustlsConfig,
        resources_manager: ResourcesManagerRef,
        cluster_manager: ClusterManagerRef,
        peer_messaging_broker: PeerMessagingBrokerRef,
        vpn: Vpn,
        carl_url: ResourceHomeUrl,
        settings: config::Config,
        ca: Pem,
        oidc_registration_client: Option<RegistrationClientRef>,
        grpc_auth_layer: GrpcAuthenticationLayer,
    ) -> BoxFuture<'static, Result<()>> {
        let oidc_enabled = settings.get_bool("network.oidc.enabled").unwrap_or(false);

        let cluster_manager_facade = ClusterManagerFacade::new(Arc::clone(&cluster_manager), Arc::clone(&resources_manager));
        let metadata_provider_facade = MetadataProviderFacade::new();

        let peer_manager_facade_options = PeerManagerFacadeOptions::load(&settings).expect("Error while loading PeerManagerFacadeOptions.");
        let peer_manager_facade = PeerManagerFacade::new(
            Arc::clone(&resources_manager),
            vpn,
            Clone::clone(&carl_url.value()),
            ca.clone(),
            oidc_registration_client,
            peer_manager_facade_options
        );
        let peer_messaging_broker_facade = PeerMessagingBrokerFacade::new(Arc::clone(&peer_messaging_broker));

        let grpc = Server::builder()
            .layer(async_interceptor(move |request| {
                Clone::clone(&grpc_auth_layer).auth_interceptor(request)
            }))
            .accept_http1(true) //gRPC-web uses HTTP1
            .add_service(cluster_manager_facade.into_grpc_service())
            .add_service(metadata_provider_facade.into_grpc_service())
            .add_service(peer_manager_facade.into_grpc_service())
            .add_service(peer_messaging_broker_facade.into_grpc_service())
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

        let carl_installation_directory = CarlInstallDirectory::determine().expect("Could not determine installation directory.");

        let app_state = HttpState {
            lea_config: LeaConfig {
                carl_url: carl_url.value(),
                idp_config: lea_idp_config,
            },
            carl_installation_directory
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

        if !project::is_running_in_development() {
            provisioning::cleo::create_cleo_install_script(
                ca,
                &app_state.carl_installation_directory.path,
                CleoScript::from_setting(&settings).expect("Could not read settings.")
            ).expect("Could not create cleo install script.");
        }

        let http = axum::Router::new()
            .fallback_service(
                axum::Router::new()
                    .nest_service(
                        "/api/licenses",
                        ServeDir::new(&licenses_dir)
                            .fallback(ServeFile::new(licenses_dir.join("index.json")))
                    )
                    .route("/api/cleo/:architecture/download", get(router::cleo::download_cleo))
                    .route("/api/edgar/:architecture/download", get(router::edgar::download_edgar))
                    .route("/api/lea/config", get(router::lea_config))
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
        oidc_registration_client,
        grpc_auth_layer,
    ).await.unwrap();

    Ok(())
}
