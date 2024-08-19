use std::sync::Arc;
use tracing::{debug, error, info};
pub use opendut_carl_api::carl::cluster::{
    CreateClusterConfigurationError,
    DeleteClusterConfigurationError,
    StoreClusterDeploymentError,
    DeleteClusterDeploymentError,
};
use opendut_types::cluster::{ClusterConfiguration, ClusterDeployment, ClusterId, ClusterName};
use opendut_types::peer::PeerId;
use crate::actions;
use crate::actions::{GetPeerStateParams, ListPeerDescriptorsParams, UnassignClusterParams};
use crate::cluster::manager::{ClusterManager, ClusterManagerRef};
use crate::resources::manager::{ResourcesManager, ResourcesManagerRef};
use crate::vpn::Vpn;

pub struct CreateClusterConfigurationParams {
    pub resources_manager: ResourcesManagerRef,
    pub cluster_configuration: ClusterConfiguration,
}

#[tracing::instrument(skip(params), level="trace")]
pub async fn create_cluster_configuration(params: CreateClusterConfigurationParams) -> Result<ClusterId, CreateClusterConfigurationError> {

    async fn inner(params: CreateClusterConfigurationParams) -> Result<ClusterId, CreateClusterConfigurationError> {

        let cluster_id = params.cluster_configuration.id;
        let cluster_name = Clone::clone(&params.cluster_configuration.name);
        let resources_manager = params.resources_manager;

        debug!("Creating cluster configuration '{cluster_name}' <{cluster_id}>.");

        resources_manager.resources_mut(|resources| {
            resources.insert(cluster_id, params.cluster_configuration)
                .map_err(|cause| CreateClusterConfigurationError::Internal { cluster_id, cluster_name: cluster_name.clone(), cause: cause.to_string() })
        }).await?;

        info!("Successfully created cluster configuration '{cluster_name}' <{cluster_id}>.");

        Ok(cluster_id)
    }

    inner(params).await
        .inspect_err(|err| error!("{err}"))
}

pub struct DeleteClusterConfigurationParams {
    pub resources_manager: ResourcesManagerRef,
    pub cluster_id: ClusterId,
}

#[tracing::instrument(skip(params), level="trace")]
pub async fn delete_cluster_configuration(params: DeleteClusterConfigurationParams) -> Result<ClusterConfiguration, DeleteClusterConfigurationError> {

    async fn inner(params: DeleteClusterConfigurationParams) -> Result<ClusterConfiguration, DeleteClusterConfigurationError> {

        let cluster_id = params.cluster_id;
        let resources_manager = params.resources_manager;

        debug!("Deleting cluster configuration <{cluster_id}>.");

        let cluster_configuration = resources_manager.resources_mut(|resources| {
            resources.remove::<ClusterConfiguration>(cluster_id)
                .map_err(|cause| DeleteClusterConfigurationError::Internal { cluster_id, cluster_name: None, cause: cause.to_string() })?
                .ok_or_else(|| DeleteClusterConfigurationError::ClusterConfigurationNotFound { cluster_id })
        }).await?;

        let cluster_name = Clone::clone(&cluster_configuration.name);

        info!("Successfully deleted cluster configuration '{cluster_name}' <{cluster_id}>.");

        Ok(cluster_configuration)
    }

    inner(params).await
        .inspect_err(|err| error!("{err}"))
}

pub struct StoreClusterConfigurationParams {
    pub cluster_manager: ClusterManagerRef,
    pub deployment: ClusterDeployment,
}

#[tracing::instrument(skip(params), level="trace")]
pub async fn store_cluster_deployment(params: StoreClusterConfigurationParams) -> Result<ClusterId, StoreClusterDeploymentError> {

    async fn inner(params: StoreClusterConfigurationParams) -> Result<ClusterId, StoreClusterDeploymentError> {
        let deployment = params.deployment;
        let mut cluster_manager = params.cluster_manager.lock().await;
        let resources_manager = cluster_manager.resources_manager().await;
        let cluster_id = deployment.id;

        resources_manager.resources_mut(|resources| {
            let cluster_name = resources.get::<ClusterConfiguration>(cluster_id)
                .map_err(|cause| StoreClusterDeploymentError::Internal { cluster_id, cluster_name: None, cause: cause.to_string() })?
                .map(|cluster| cluster.name)
                .unwrap_or_else(|| ClusterName::try_from("unknown_cluster").unwrap());
            resources.insert(cluster_id, deployment)
                .map_err(|cause| StoreClusterDeploymentError::Internal { cluster_id, cluster_name: Some(cluster_name.clone()), cause: cause.to_string() })
        }).await?;
        if let Err(error) = cluster_manager.deploy(cluster_id).await {
            error!("Failed to deploy cluster <{cluster_id}>, due to:\n  {error}");
        }
        Ok(cluster_id)
    }

    inner(params).await
        .inspect_err(|err| error!("{err}"))
}

pub struct DeleteClusterDeploymentParams {
    pub cluster_manager: ClusterManagerRef,
    pub cluster_id: ClusterId,
}

#[tracing::instrument(skip(params), level="trace")]
pub async fn delete_cluster_deployment(params: DeleteClusterDeploymentParams) -> Result<ClusterDeployment, DeleteClusterDeploymentError> {

    async fn inner(params: DeleteClusterDeploymentParams) -> Result<ClusterDeployment, DeleteClusterDeploymentError> {
        let cluster_manager = params.cluster_manager.lock().await;
        let cluster_id = params.cluster_id;

        let (deployment, configuration) = cluster_manager.resources_manager().await
            .resources_mut(|resources| {
                resources.remove::<ClusterDeployment>(cluster_id)
                    .map_err(|cause| DeleteClusterDeploymentError::Internal { cluster_id, cluster_name: None, cause: cause.to_string() })?
                    .map(|deployment| {
                        let configuration = resources.get::<ClusterConfiguration>(cluster_id)
                            .map_err(|cause| DeleteClusterDeploymentError::Internal { cluster_id, cluster_name: None, cause: cause.to_string() })?;
                        Ok((deployment, configuration))
                    }).transpose()
            }).await?
            .ok_or(DeleteClusterDeploymentError::ClusterDeploymentNotFound { cluster_id })?;

        if let Some(configuration) = configuration {
            if let Vpn::Enabled { vpn_client } = cluster_manager.vpn().await {
                vpn_client.delete_cluster(cluster_id).await
                    .map_err(|error| DeleteClusterDeploymentError::Internal { cluster_id, cluster_name: Some(configuration.name.clone()), cause: error.to_string() })?;
            }

            {
                let all_peers = actions::list_peer_descriptors(ListPeerDescriptorsParams {
                    resources_manager: Arc::clone(&cluster_manager.resources_manager().await),
                }).await.map_err(|cause| DeleteClusterDeploymentError::Internal { cluster_id, cluster_name: Some(configuration.name.clone()), cause: cause.to_string() })?;

                let member_ids = all_peers.into_iter()
                    .filter(|peer| !peer.topology.devices.iter().filter(|device| configuration.devices.contains(&device.id)).collect::<Vec<_>>().is_empty() )
                    .map(|peer| peer.id).collect::<Vec<PeerId>>();

                for member_id in member_ids {
                    actions::unassign_cluster(UnassignClusterParams {
                        resources_manager: Arc::clone(&cluster_manager.resources_manager().await),
                        peer_id: member_id,
                    }).await
                        .map_err(|cause| {
                            let message = format!("Failure while assigning cluster <{cluster_id}> to peer <{member_id}>.");
                            error!("{}\n  {cause}", message);
                            DeleteClusterDeploymentError::Internal { cluster_id, cluster_name: Some(configuration.name.clone()), cause: message }
                        })?;
                }
            }

        }

        Ok(deployment)
    }

    inner(params).await
        .inspect_err(|err| error!("{err}"))
}
