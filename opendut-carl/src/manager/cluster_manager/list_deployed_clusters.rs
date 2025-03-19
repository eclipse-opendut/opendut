use crate::resource::persistence::error::{PersistenceError};
use crate::resource::storage::ResourcesStorageApi;
use opendut_types::cluster::{ClusterConfiguration, ClusterDeployment};
use std::collections::{HashSet};

pub mod internal {
    use super::*;

    pub(crate) fn list_deployed_clusters(resources: &impl ResourcesStorageApi) -> Result<Vec<ClusterConfiguration>, PersistenceError> {
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
    }
}
