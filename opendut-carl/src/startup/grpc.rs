use std::sync::Arc;
use anyhow::Context;
use config::Config;
use pem::Pem;
use opendut_auth::registration::client::RegistrationClientRef;
use opendut_auth::registration::resources::ResourceHomeUrl;
use crate::manager::grpc::{ClusterManagerFacade, MetadataProviderFacade, PeerManagerFacade, PeerMessagingBrokerFacade};
use crate::resource::manager::ResourceManager;
use crate::resource::storage::PersistenceOptions;
use crate::startup;
use crate::manager::cluster_manager::{ClusterManager, ClusterManagerOptions};
use crate::manager::peer_messaging_broker::{PeerMessagingBroker, PeerMessagingBrokerOptions};
use crate::resource::api::global::GlobalResources;
use crate::settings::vpn;

pub struct GrpcFacades {
    pub cluster_manager_facade: ClusterManagerFacade,
    pub metadata_provider_facade: MetadataProviderFacade,
    pub peer_manager_facade: PeerManagerFacade,
    pub peer_messaging_broker_facade: PeerMessagingBrokerFacade,
}

impl GrpcFacades {
    pub async fn create(
        carl_url: &ResourceHomeUrl,
        ca_certificate: Pem,
        oidc_registration_client: Option<RegistrationClientRef>,
        settings: &Config,
    ) -> anyhow::Result<Self> {

        let vpn = vpn::create(settings).await
            .context("Error while parsing VPN configuration.")?;

        let resource_manager = {
            let mut global = GlobalResources::default();
            global.insert(vpn.clone());
            let global = global.complete();

            let resources_storage_options = PersistenceOptions::load(settings)?;

            ResourceManager::create(global, resources_storage_options).await
                .context("Creating ResourceManager failed")?
        };

        startup::metrics::initialize_metrics_collection(Arc::clone(&resource_manager));

        let peer_messaging_broker = PeerMessagingBroker::new(
            Arc::clone(&resource_manager),
            PeerMessagingBrokerOptions::load(settings)?,
        );
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

        Ok(GrpcFacades {
            cluster_manager_facade,
            metadata_provider_facade,
            peer_manager_facade,
            peer_messaging_broker_facade,
        })
    }
}
