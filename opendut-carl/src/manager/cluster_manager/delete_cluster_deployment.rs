use opendut_types::ShortName;
use opendut_types::cluster::ClusterDisplay;
use crate::settings::vpn::Vpn;
use opendut_types::cluster::{ClusterDescriptor, ClusterDeployment, ClusterId, ClusterName};
use opendut_types::cluster::state::ClusterState;
use crate::resource::api::resources::Resources;
use crate::resource::persistence::error::PersistenceError;
use crate::resource::storage::ResourcesStorageApi;

pub struct DeleteClusterDeploymentParams {
    pub cluster_id: ClusterId,
    pub vpn: Vpn,
}

impl Resources<'_> {
    #[tracing::instrument(skip_all, level="trace")]
    pub async fn delete_cluster_deployment(&mut self, params: DeleteClusterDeploymentParams) -> Result<ClusterDeployment, DeleteClusterDeploymentError> {

        let DeleteClusterDeploymentParams { cluster_id, vpn } = params;

        let (deployment, cluster) =
            self.remove::<ClusterDeployment>(cluster_id)
                .map_err(|source| DeleteClusterDeploymentError::Persistence { cluster_id, cluster_name: None, source })?
                .map(|deployment| {
                    let configuration = self.get::<ClusterDescriptor>(cluster_id)
                        .map_err(|source| DeleteClusterDeploymentError::Persistence { cluster_id, cluster_name: None, source })?;
                    Ok((deployment, configuration))
                })
                .ok_or(DeleteClusterDeploymentError::ClusterDeploymentNotFound { cluster_id })??;

        if let Some(cluster) = cluster {
            if let Vpn::Enabled { vpn_client } = vpn {
                vpn_client.delete_cluster(cluster_id).await
                    .map_err(|source| DeleteClusterDeploymentError::VpnClient { cluster_id, cluster_name: cluster.name.clone(), source })?;
            }

            // TODO: unassign cluster for each peer
        }

        Ok(deployment)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum DeleteClusterDeploymentError {
    #[error("ClusterDeployment for cluster <{cluster_id}> could not be deleted, because a ClusterDeployment with that id does not exist!")]
    ClusterDeploymentNotFound {
        cluster_id: ClusterId
    },
    #[error(
        "ClusterDeployment for cluster '{cluster_name}' <{cluster_id}> cannot be deleted when cluster is in state '{actual_state}'! A peer can be deleted when: {required_states}",
        actual_state = actual_state.short_name(),
        required_states = ClusterState::short_names_joined(required_states)
    )]
    IllegalClusterState {
        cluster_id: ClusterId,
        cluster_name: ClusterName,
        actual_state: ClusterState,
        required_states: Vec<ClusterState>,
    },
    #[error("Error when accessing persistence while deleting cluster deployment for cluster {cluster}", cluster=ClusterDisplay::new(cluster_name, cluster_id))]
    Persistence {
        cluster_id: ClusterId,
        cluster_name: Option<ClusterName>,
        #[source] source: PersistenceError,
    },
    #[error("Error when deleting cluster in VPN management service while deleting cluster deployment for cluster {cluster}", cluster=ClusterDisplay::new(&Some(cluster_name.to_owned()), cluster_id))]
    VpnClient {
        cluster_id: ClusterId,
        cluster_name: ClusterName,
        #[source] source: opendut_vpn::DeleteClusterError,
    }
}
