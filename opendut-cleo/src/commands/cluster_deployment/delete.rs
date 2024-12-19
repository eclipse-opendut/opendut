use opendut_carl_api::carl::CarlClient;
use opendut_types::cluster::ClusterId;

/// Delete a cluster deployment
#[derive(clap::Parser)]
pub struct DeleteClusterDeploymentCli {
    /// ID of the cluster
    #[arg()]
    id: ClusterId,
}

impl DeleteClusterDeploymentCli {
    pub async fn execute(self, carl: &mut CarlClient) -> crate::Result<()> {
        let id = self.id;
        carl.cluster.delete_cluster_deployment(id).await
            .map_err(|error| {
                format!("Could not delete cluster deployment for ClusterID '{}'.\n  {}", id, error)
            })?;
        println!("Deleted cluster deployment for ClusterID '{}'.", id);

        Ok(())
    }
}
