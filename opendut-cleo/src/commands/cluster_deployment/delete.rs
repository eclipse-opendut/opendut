use opendut_carl_api::carl::cluster::DeleteClusterDeploymentError;
use opendut_carl_api::carl::{CarlClient, ClientError};
use opendut_model::cluster::ClusterId;

/// Delete a cluster deployment
#[derive(clap::Parser)]
pub struct DeleteClusterDeploymentCli {
    /// ID of the cluster
    #[arg()]
    id: ClusterId,
    /// Report an error status code when the resource to delete does not exist
    #[arg(long)]
    error_when_missing: bool,
}

impl DeleteClusterDeploymentCli {
    pub async fn execute(self, carl: &mut CarlClient) -> crate::Result<()> {
        let id = self.id;

        match carl.cluster.delete_cluster_deployment(id).await {
            Ok(_) => Ok(()),
            Err(error) => match error {
                ClientError::UsageError(DeleteClusterDeploymentError::ClusterDeploymentNotFound { .. }) => {
                    eprintln!("No cluster deployment found with ID <{id}>.");
                    if self.error_when_missing {
                        Err(error)
                    } else {
                        Ok(())
                    }
                }
                other => Err(other)
            }
            .map_err(|error| format!("Could not delete cluster deployment for ClusterID '{id}'.\n  {error}"))
        }?;

        println!("Deleted cluster deployment for ClusterID '{id}'.");

        Ok(())
    }
}
