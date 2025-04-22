use opendut_types::cluster::{ClusterConfiguration, ClusterId, ClusterName};
use tracing::{debug, info};
use crate::resource::api::resources::Resources;
use crate::resource::persistence::error::PersistenceError;
use crate::resource::storage::ResourcesStorageApi;

pub struct CreateClusterConfigurationParams {
    pub cluster_configuration: ClusterConfiguration,
}

impl Resources<'_> {
    #[tracing::instrument(skip_all, level="trace")]
    pub fn create_cluster_configuration(&mut self, params: CreateClusterConfigurationParams) -> Result<ClusterId, CreateClusterConfigurationError> {

        let cluster_id = params.cluster_configuration.id;
        let cluster_name = Clone::clone(&params.cluster_configuration.name);

        debug!("Creating cluster configuration '{cluster_name}' <{cluster_id}>.");

        self.insert(cluster_id, params.cluster_configuration)
            .map_err(|source| CreateClusterConfigurationError::Persistence { cluster_id, cluster_name: cluster_name.clone(), source })?;

        info!("Successfully created cluster configuration '{cluster_name}' <{cluster_id}>.");

        Ok(cluster_id)
    }
}

#[derive(thiserror::Error, Debug)]
#[error("ClusterConfigration '{cluster_name}' <{cluster_id}> could not be created")]
pub enum CreateClusterConfigurationError {
    Persistence {
        cluster_id: ClusterId,
        cluster_name: ClusterName,
        #[source] source: PersistenceError
    }
}
