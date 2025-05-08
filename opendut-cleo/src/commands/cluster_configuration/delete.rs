use opendut_carl_api::carl::cluster::DeleteClusterConfigurationError;
use opendut_carl_api::carl::{CarlClient, ClientError};
use opendut_types::cluster::ClusterId;

/// Delete a cluster configuration
#[derive(clap::Parser)]
pub struct DeleteClusterConfigurationCli {
    /// ID of the cluster
    #[arg()]
    id: ClusterId,
    /// Report an error status code when the resource to delete does not exist
    #[arg(long)]
    error_when_missing: bool,
}

impl DeleteClusterConfigurationCli {
    pub async fn execute(self, carl: &mut CarlClient) -> crate::Result<()> {
        let id = self.id;

        let cluster_deployments = carl.cluster.list_cluster_deployments().await
            .map_err(|_| String::from("Failed to get list of cluster deployments!"))?;

        if cluster_deployments.iter().any(|deployment| deployment.id == id) {
            Err(format!("Cluster <{}> can not be deleted while it is deployed.", id))?
        };
        
        match carl.cluster.delete_cluster_configuration(id).await {
            Ok(cluster_configuration) => {
                eprintln!("Deleted ClusterConfiguration {} <{}> successfully.", cluster_configuration.name, cluster_configuration.id);
                Ok(())
            }
            Err(error) => match error {
                ClientError::UsageError(DeleteClusterConfigurationError::ClusterConfigurationNotFound { .. }) => {
                    eprintln!("No cluster configuration found with ID <{id}>.");
                    if self.error_when_missing {
                        Err(error)
                    } else {
                        Ok(())
                    }
                }
                other => Err(other),
            }
        }.map_err(|error| format!("Failed to delete ClusterConfiguration with id <{id}>.\n  {error}"))
    }
}
