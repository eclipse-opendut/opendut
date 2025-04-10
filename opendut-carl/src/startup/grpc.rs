use std::sync::Arc;
use anyhow::Context;
use config::Config;
use pem::Pem;
use opendut_auth::registration::client::RegistrationClientRef;
use opendut_auth::registration::resources::ResourceHomeUrl;
use crate::manager::grpc::{ClusterManagerFacade, MetadataProviderFacade, ObserverMessagingBrokerFacade, PeerManagerFacade, PeerMessagingBrokerFacade};
use crate::resource::manager::ResourceManager;
use crate::resource::storage::PersistenceOptions;
use crate::startup;
use crate::manager::cluster_manager::{ClusterManager, ClusterManagerOptions};
use crate::manager::observer_messaging_broker::ObserverMessagingBroker;
use crate::manager::peer_messaging_broker::{PeerMessagingBroker, PeerMessagingBrokerOptions};
use crate::settings::vpn;

pub struct GrpcFacades {
    pub cluster_manager_facade: ClusterManagerFacade,
    pub metadata_provider_facade: MetadataProviderFacade,
    pub peer_manager_facade: PeerManagerFacade,
    pub peer_messaging_broker_facade: PeerMessagingBrokerFacade,
    pub observer_messaging_broker_facade: ObserverMessagingBrokerFacade,
}

impl GrpcFacades {
    pub async fn create(
        carl_url: &ResourceHomeUrl,
        ca_certificate: Pem,
        oidc_registration_client: Option<RegistrationClientRef>,
        settings: &Config,
    ) -> anyhow::Result<Self> {

        let resource_manager = {
            let persistence_options = PersistenceOptions::load(settings)?;

            let resource_manager = ResourceManager::create(&persistence_options).await
                .context("Creating ResourceManager failed")?;

            #[cfg(feature="postgres")]
            if let Some(value) = std::env::var_os("OPENDUT_CARL_POSTGRES_MIGRATION") {
                tracing::info!("Found environment variable `OPENDUT_CARL_POSTGRES_MIGRATION`. Starting migration.");
                assert!(!value.is_empty());

                startup::postgres_migration::load_data_from_postgres_into_key_value_store(resource_manager.clone(), &persistence_options).await
                    .expect("Migration from Postgres to Key-Value Store should complete without errors");

                tracing::info!("Migration complete. Exiting.");
                std::process::exit(0);
            }

            resource_manager
        };

        let vpn = vpn::create(settings).await
            .context("Error while parsing VPN configuration.")?;

        startup::metrics::initialize_metrics_collection(Arc::clone(&resource_manager));

        let peer_messaging_broker = PeerMessagingBroker::new(
            Arc::clone(&resource_manager),
            PeerMessagingBrokerOptions::load(settings)?,
        ).await;
        let cluster_manager = ClusterManager::create(
            Arc::clone(&resource_manager),
            Arc::clone(&peer_messaging_broker),
            Clone::clone(&vpn),
            ClusterManagerOptions::load(settings)?,
        ).await;


        let cluster_manager_facade = ClusterManagerFacade::new(Arc::clone(&cluster_manager), Arc::clone(&resource_manager));

        let metadata_provider_facade = MetadataProviderFacade::new();

        let peer_manager_facade = PeerManagerFacade::new(
            Arc::clone(&resource_manager),
            vpn,
            Clone::clone(&carl_url.value()),
            ca_certificate,
            oidc_registration_client,
        );
        let peer_messaging_broker_facade = PeerMessagingBrokerFacade::new(Arc::clone(&peer_messaging_broker));
        
        let observer_messaging_broker = ObserverMessagingBroker::new(Arc::clone(&resource_manager), Arc::clone(&cluster_manager));
        let observer_messaging_broker_facade = ObserverMessagingBrokerFacade::new(Arc::clone(&resource_manager), Arc::clone(&observer_messaging_broker));

        Ok(GrpcFacades {
            cluster_manager_facade,
            metadata_provider_facade,
            peer_manager_facade,
            peer_messaging_broker_facade,
            observer_messaging_broker_facade,
        })
    }
}
