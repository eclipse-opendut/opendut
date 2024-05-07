use uuid::Uuid;
use opendut_carl_api::carl::{CarlClient};
use opendut_types::cluster::ClusterId;

/// Delete a cluster configuration
#[derive(clap::Parser)]
pub struct DeleteClusterConfigurationCli {
    ///ClusterID
    #[arg()]
    id: Uuid,
}

impl DeleteClusterConfigurationCli {
    pub async fn execute(self, carl: &mut CarlClient) -> crate::Result<()> {
        let id = ClusterId::from(self.id);
        let cluster_configuration = carl.cluster.delete_cluster_configuration(id).await
            .map_err(|error| format!("Failed to delete ClusterConfiguration with id <{id}>.\n  {error}"))?;

        println!("Deleted ClusterConfiguration {} <{}> successfully.", cluster_configuration.name, cluster_configuration.id);

        Ok(())
    }
}
