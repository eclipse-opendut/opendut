use std::net::SocketAddr;
use std::str::FromStr;

use pem::Pem;
use tonic_async_interceptor::async_interceptor;
use tower::ServiceExt;
use tracing::info;
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
use crate::http::state::CarlInstallDirectory;
use crate::provisioning::cleo_script::CleoScript;
use crate::startup::tls::TlsConfig;
use startup::multiplex_service::GrpcHttpMultiplexLayer;

pub mod grpc;
pub mod util;
opendut_util::app_info!();

mod actions;
mod cluster;
pub mod persistence;
mod peer;
mod resources;
pub mod settings;
mod startup;
mod vpn;
mod http;
mod provisioning;
mod auth;

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

    let carl_url = ResourceHomeUrl::try_from(&settings)?;

    let ca_certificate = Pem::from_config_path("network.tls.ca", &settings).await?;

    let oidc_registration_client = RegistrationClient::from_settings(&settings).await
        .expect("Failed to load oidc registration client!");

    let grpc_facades = startup::grpc::GrpcFacades::create(
        &carl_url,
        ca_certificate.clone(),
        oidc_registration_client.clone(),
        &settings
    ).await?;

    let http = {
        let carl_installation_directory = CarlInstallDirectory::determine()
            .expect("Could not determine installation directory.");

        if !project::is_running_in_development() {
            provisioning::cleo::create_cleo_install_script(
                ca_certificate,
                &carl_installation_directory.path,
                CleoScript::from_setting(&settings).expect("Could not read settings.")
            ).expect("Could not create CLEO install script.");
        }

        let http_state = startup::http::create_http_state(&carl_url, carl_installation_directory, &settings)?;

        startup::http::create_http_service(&settings)?
            .with_state(http_state)
            .into_service()
            .map_response(|response| -> ::http::Response<tonic::body::BoxBody> { response.map(tonic::body::boxed) })
    };

    let grpc = {
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

        let grpc = tonic::transport::Server::builder()
            .layer(async_interceptor(move |request| {
                Clone::clone(&grpc_auth_layer).auth_interceptor(request)
            }))
            .accept_http1(true) //gRPC-web uses HTTP1
            .layer(GrpcHttpMultiplexLayer::new_with_http(http));


        let mut grpc =
            if let TlsConfig::Enabled(tls_config) = TlsConfig::load(&settings)? {
                opendut_util::crypto::install_default_provider();
                grpc.tls_config(tls_config)?
            } else {
                info!("TLS is disabled in the configuration.");
                grpc
            };

        grpc
            .add_service(grpc_facades.cluster_manager_facade.into_grpc_service())
            .add_service(grpc_facades.metadata_provider_facade.into_grpc_service())
            .add_service(grpc_facades.peer_manager_facade.into_grpc_service())
            .add_service(grpc_facades.peer_messaging_broker_facade.into_grpc_service())
    };


    let address: SocketAddr = {
        let host = settings.get_string("network.bind.host")?;
        let port = settings.get_int("network.bind.port")?;
        SocketAddr::from_str(&format!("{host}:{port}"))?
    };

    info!("Server listening at {address}...");
    grpc.serve(address).await?;

    Ok(())
}
