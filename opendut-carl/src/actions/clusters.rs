use crate::actions;
use crate::actions::{GetPeerStateParams, ListPeerDescriptorsParams, UnassignClusterParams};
use crate::cluster::manager::{ClusterManager, ClusterManagerRef};
use crate::resources::manager::{ResourcesManager, ResourcesManagerRef};
use crate::vpn::Vpn;
pub use opendut_carl_api::carl::cluster::{
    CreateClusterConfigurationError,
    DeleteClusterConfigurationError,
    DeleteClusterDeploymentError,
    StoreClusterDeploymentError,
};
use opendut_types::cluster::{ClusterConfiguration, ClusterDeployment, ClusterId, ClusterName};
use opendut_types::peer::{PeerDescriptor, PeerId};
use std::ops::Not;
use std::sync::Arc;
use tracing::{debug, error, info};

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
    pub resources_manager: ResourcesManagerRef,
    pub deployment: ClusterDeployment,
}

#[tracing::instrument(skip(params), level="trace")]
pub async fn store_cluster_deployment(params: StoreClusterConfigurationParams) -> Result<ClusterId, StoreClusterDeploymentError> {

    async fn inner(params: StoreClusterConfigurationParams) -> Result<ClusterId, StoreClusterDeploymentError> {
        let StoreClusterConfigurationParams { resources_manager, deployment } = params;
        let cluster_id = deployment.id;

        resources_manager.resources_mut(|resources| {
            let cluster_name = resources.get::<ClusterConfiguration>(cluster_id)
                .map_err(|cause| StoreClusterDeploymentError::Internal { cluster_id, cluster_name: None, cause: cause.to_string() })?
                .map(|cluster| cluster.name)
                .unwrap_or_else(|| ClusterName::try_from("unknown_cluster").unwrap());
            resources.insert(cluster_id, deployment)
                .map_err(|cause| StoreClusterDeploymentError::Internal { cluster_id, cluster_name: Some(cluster_name.clone()), cause: cause.to_string() })
        }).await?;

        Ok(cluster_id)
    }

    inner(params).await
        .inspect_err(|err| error!("{err}"))
}

pub struct DeleteClusterDeploymentParams {
    pub resources_manager: ResourcesManagerRef,
    pub vpn: Vpn,
    pub cluster_id: ClusterId,
}

#[tracing::instrument(skip(params), level="trace")]
pub async fn delete_cluster_deployment(params: DeleteClusterDeploymentParams) -> Result<ClusterDeployment, DeleteClusterDeploymentError> {

    async fn inner(params: DeleteClusterDeploymentParams) -> Result<ClusterDeployment, DeleteClusterDeploymentError> {
        let DeleteClusterDeploymentParams { resources_manager, vpn, cluster_id } = params;

        let (deployment, cluster) = resources_manager
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

        if let Some(cluster) = cluster {
            if let Vpn::Enabled { vpn_client } = vpn {
                vpn_client.delete_cluster(cluster_id).await
                    .map_err(|error| DeleteClusterDeploymentError::Internal { cluster_id, cluster_name: Some(cluster.name.clone()), cause: error.to_string() })?;
            }

            {
                let all_peers = resources_manager.list::<PeerDescriptor>().await
                    .map_err(|cause| DeleteClusterDeploymentError::Internal { cluster_id, cluster_name: Some(cluster.name.clone()), cause: cause.to_string() })?;

                let member_ids = all_peers.into_iter()
                    .filter(|peer|
                        peer.topology.devices.iter()
                            .filter(|device| cluster.devices.contains(&device.id))
                            .collect::<Vec<_>>()
                            .is_empty().not()
                    )
                    .map(|peer| peer.id)
                    .collect::<Vec<PeerId>>();

                for member_id in member_ids {
                    actions::unassign_cluster(UnassignClusterParams {
                        resources_manager: Arc::clone(&resources_manager),
                        peer_id: member_id,
                    }).await
                        .map_err(|cause| {
                            let message = format!("Failure while assigning cluster <{cluster_id}> to peer <{member_id}>.");
                            error!("{}\n  {cause}", message);
                            DeleteClusterDeploymentError::Internal { cluster_id, cluster_name: Some(cluster.name.clone()), cause: message }
                        })?;
                }
            }
        }

        Ok(deployment)
    }

    inner(params).await
        .inspect_err(|err| error!("{err}"))
}
