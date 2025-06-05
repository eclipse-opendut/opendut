use opendut_types::cluster::{ClusterDescriptor, ClusterId, ClusterName};
use tracing::{debug, info};
use crate::resource::api::resources::Resources;
use crate::resource::persistence::error::PersistenceError;
use crate::resource::storage::ResourcesStorageApi;

pub struct CreateClusterDescriptorParams {
    pub cluster_descriptor: ClusterDescriptor,
}

impl Resources<'_> {
    #[tracing::instrument(skip_all, level="trace")]
    pub fn create_cluster_descriptor(&mut self, params: CreateClusterDescriptorParams) -> Result<ClusterId, CreateClusterDescriptorError> {

        let cluster_id = params.cluster_descriptor.id;
        let cluster_name = Clone::clone(&params.cluster_descriptor.name);

        debug!("Creating cluster descriptor '{cluster_name}' <{cluster_id}>.");

        self.insert(cluster_id, params.cluster_descriptor)
            .map_err(|source| CreateClusterDescriptorError::Persistence { cluster_id, cluster_name: cluster_name.clone(), source })?;

        info!("Successfully created cluster descriptor '{cluster_name}' <{cluster_id}>.");

        Ok(cluster_id)
    }
}

#[derive(thiserror::Error, Debug)]
#[error("ClusterConfigration '{cluster_name}' <{cluster_id}> could not be created")]
pub enum CreateClusterDescriptorError {
    Persistence {
        cluster_id: ClusterId,
        cluster_name: ClusterName,
        #[source] source: PersistenceError
    }
}
