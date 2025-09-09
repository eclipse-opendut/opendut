use crate::resource::persistence::error::{PersistenceError};
use crate::resource::storage::ResourcesStorageApi;
use opendut_model::cluster::{ClusterDescriptor, ClusterDeployment};
use std::collections::{HashSet};

pub mod internal {
    use super::*;

    pub(crate) fn list_deployed_clusters(resources: &impl ResourcesStorageApi) -> Result<Vec<ClusterDescriptor>, PersistenceError> {
        let cluster_deployments = resources.list::<ClusterDeployment>()?
            .into_values()
            .map(|cluster_deployment| cluster_deployment.id)
            .collect::<HashSet<_>>();

        let cluster_descriptors = resources.list::<ClusterDescriptor>()?;
        let deployed_cluster_descriptors = cluster_descriptors.into_values()
            .filter(|cluster_descriptor| {
                cluster_deployments.contains(&cluster_descriptor.id)
            }).collect::<Vec<_>>();
        Ok(deployed_cluster_descriptors)
    }
}
