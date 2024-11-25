extern crate core;

use std::fs;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;

use anyhow::{anyhow, Context};
use axum::routing::get;
use pem::Pem;
use tonic::transport::{Identity, ServerTlsConfig};
use tonic_async_interceptor::async_interceptor;
use tower::ServiceExt;
use tower_http::services::{ServeDir, ServeFile};
use tracing::{debug, info};
use uuid::Uuid;

use opendut_auth::confidential::pem::PemFromConfig;
use opendut_auth::registration::client::RegistrationClient;
use opendut_auth::registration::resources::ResourceHomeUrl;
use opendut_util::settings::LoadedConfig;
use opendut_util::telemetry::logging::LoggingConfig;
use opendut_util::telemetry::opentelemetry_types::Opentelemetry;
use opendut_util::{project, telemetry};
use util::in_memory_cache::CustomInMemoryCache;

use crate::auth::grpc_auth_layer::GrpcAuthenticationLayer;
use crate::auth::json_web_key::JwkCacheValue;
use crate::cluster::manager::{ClusterManager, ClusterManagerOptions};
use crate::grpc::{ClusterManagerFacade, MetadataProviderFacade, PeerManagerFacade, PeerMessagingBrokerFacade};
use crate::http::router;
use crate::http::state::{CarlInstallDirectory, HttpState, LeaConfig, LeaIdentityProviderConfig};
use crate::multiplex_service::GrpcHttpMultiplexLayer;
use crate::peer::broker::{PeerMessagingBroker, PeerMessagingBrokerOptions};
use crate::provisioning::cleo_script::CleoScript;
use crate::resources::manager::ResourcesManager;
use crate::resources::storage::PersistenceOptions;

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
mod multiplex_service;

#[tracing::instrument]
pub async fn create_with_telemetry(settings_override: config::Config) -> anyhow::Result<()> {
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

pub async fn create(settings: LoadedConfig) -> anyhow::Result<()> {
    info!("Started with configuration: {settings:?}");
    let settings = settings.config;

    let address: SocketAddr = {
        let host = settings.get_string("network.bind.host")?;
        let port = settings.get_int("network.bind.port")?;
        SocketAddr::from_str(&format!("{host}:{port}"))?
    };

    let tls_config = {
        let tls_enabled: bool = settings.get_bool("network.tls.enabled")
            .map_err(|cause| anyhow!("Expected configuration flag 'network.tls.enabled' to be parseable as boolean! {}", cause))?;

        if tls_enabled {
            let cert = {
                let cert_path = project::make_path_absolute(settings.get_string("network.tls.certificate")?)?;
                debug!("Using TLS certificate: {}", cert_path.display());
                assert!(cert_path.exists(), "TLS certificate file at '{}' not found.", cert_path.display());
                fs::read(&cert_path)
                    .context(format!("Error while reading TLS certificate at {}", cert_path.display()))?
            };

            let key = {
                let key_path = project::make_path_absolute(settings.get_string("network.tls.key")?)?;
                debug!("Using TLS key: {}", key_path.display());
                assert!(key_path.exists(), "TLS key file at '{}' not found.", key_path.display());
                fs::read(&key_path)
                    .context(format!("Error while reading TLS key at {}", key_path.display()))?
            };

            let tls_config = ServerTlsConfig::new()
                .identity(Identity::from_pem(cert, key));

            TlsConfig::Enabled(tls_config)
        } else {
            TlsConfig::Disabled
        }
    };

    let carl_url = ResourceHomeUrl::try_from(&settings)?;

    let ca_certificate = Pem::from_config_path("network.tls.ca", &settings).await?;
    let oidc_registration_client = RegistrationClient::from_settings(&settings).await.expect("Failed to load oidc registration client!");

    let vpn = vpn::create(&settings)
        .context("Error while parsing VPN configuration.")?;

    let resources_manager = {
        let resources_storage_options = PersistenceOptions::load(&settings)?;

        ResourcesManager::create(resources_storage_options).await
            .context("Creating ResourcesManager failed")?
    };

    metrics::initialize_metrics_collection(Arc::clone(&resources_manager));

    let peer_messaging_broker = PeerMessagingBroker::new(
        Arc::clone(&resources_manager),
        PeerMessagingBrokerOptions::load(&settings)?,
    );
    let cluster_manager = ClusterManager::create(
        Arc::clone(&resources_manager),
        Arc::clone(&peer_messaging_broker),
        Clone::clone(&vpn),
        ClusterManagerOptions::load(&settings)?,
    ).await;


    let oidc_enabled = settings.get_bool("network.oidc.enabled").unwrap_or(false);

    let cluster_manager_facade = ClusterManagerFacade::new(Arc::clone(&cluster_manager), Arc::clone(&resources_manager));
    let metadata_provider_facade = MetadataProviderFacade::new();

    let peer_manager_facade = PeerManagerFacade::new(
        Arc::clone(&resources_manager),
        vpn,
        Clone::clone(&carl_url.value()),
        ca_certificate.clone(),
        oidc_registration_client.clone(),
    );
    let peer_messaging_broker_facade = PeerMessagingBrokerFacade::new(Arc::clone(&peer_messaging_broker));

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
            ca_certificate,
            &app_state.carl_installation_directory.path,
            CleoScript::from_setting(&settings).expect("Could not read settings.")
        ).expect("Could not create cleo install script.");
    }

    let http = axum::Router::new()
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
        .into_service()
        .map_response(|response| response.map(tonic::body::boxed));

    let grpc_auth_layer = match oidc_registration_client {
        None => GrpcAuthenticationLayer::AuthDisabled,
        Some(oidc_client_ref) => {
            let jwk_cache: CustomInMemoryCache<String, JwkCacheValue> = CustomInMemoryCache::new();

            GrpcAuthenticationLayer::GrpcAuthLayerEnabled {
                issuer_url: oidc_client_ref.inner.config.issuer_url.clone(),
                issuer_remote_url: oidc_client_ref.config.issuer_remote_url.clone(),
                cache: jwk_cache,
            }
        }
    };

    let grpc = {
        let grpc = tonic::transport::Server::builder()
            .layer(async_interceptor(move |request| {
                Clone::clone(&grpc_auth_layer).auth_interceptor(request)
            }))
            .accept_http1(true) //gRPC-web uses HTTP1
            .layer(GrpcHttpMultiplexLayer::new_with_http(http));

        let mut grpc = if let TlsConfig::Enabled(tls_config) = tls_config {
            grpc.tls_config(tls_config)?
        } else {
            grpc
        };

        grpc
            .add_service(cluster_manager_facade.into_grpc_service())
            .add_service(metadata_provider_facade.into_grpc_service())
            .add_service(peer_manager_facade.into_grpc_service())
            .add_service(peer_messaging_broker_facade.into_grpc_service())
    };

    info!("Server listening at {address}...");
    grpc.serve(address).await?;

    Ok(())
}

enum TlsConfig {
    Enabled(ServerTlsConfig),
    Disabled
}
