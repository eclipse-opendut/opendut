use crate::persistence::error::{PersistenceError};
use crate::resources::manager::ResourcesManagerRef;
use crate::resources::storage::ResourcesStorageApi;
use opendut_types::cluster::{ClusterConfiguration, ClusterDeployment};
use std::collections::{HashSet};

pub struct ListDeployedClustersParams {
    pub resources_manager: ResourcesManagerRef,
}

#[derive(thiserror::Error, Debug)]
pub enum ListDeployedClustersError {
    #[error("Error fetching deployed clusters from persistence.")]
    Persistence { #[source] source: PersistenceError },
}

#[tracing::instrument(skip(params), level="trace")]
pub async fn list_deployed_clusters(params: ListDeployedClustersParams) -> Result<Vec<ClusterConfiguration>, ListDeployedClustersError> {
    
    
    let deployed_cluster_configurations = params.resources_manager.resources(|resources| {
        let cluster_deployments = resources.list::<ClusterDeployment>()?
            .into_iter()
            .map(|cluster_deployment| cluster_deployment.id)
            .collect::<HashSet<_>>();
        
        let cluster_configurations = resources.list::<ClusterConfiguration>()?;
        let deployed_cluster_configurations = cluster_configurations.into_iter()
            .filter(|cluster_configuration| {
                cluster_deployments.contains(&cluster_configuration.id)
            }).collect::<Vec<_>>();

        Ok(deployed_cluster_configurations)
    }).await
    .map_err(|source| ListDeployedClustersError::Persistence { source })?;

    Ok(deployed_cluster_configurations)
}
