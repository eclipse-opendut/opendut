use crate::CreateOutputFormat;
use opendut_carl_api::carl::CarlClient;
use opendut_types::cluster::{ClusterDeployment, ClusterId};

/// Create a cluster deployment
#[derive(clap::Parser)]
pub struct CreateClusterDeploymentCli {
    /// ID of the cluster
    #[arg()]
    id: ClusterId,
}

impl CreateClusterDeploymentCli {
    pub async fn execute(self, carl: &mut CarlClient, output: CreateOutputFormat) -> crate::Result<()> {
        let id = self.id;

        let deployment = ClusterDeployment { id };
        carl.cluster.store_cluster_deployment(deployment).await
            .map_err(|error| format!("Could not create cluster deployment for ClusterID: '{}'.\n  {}", id, error))?;
        match output {
            CreateOutputFormat::Text => {
                println!("Successfully created cluster deployment for cluster <{}>.", id);
            }
            CreateOutputFormat::Json => {
                let json = serde_json::to_string(&id).unwrap();
                println!("{}", json);
            }
            CreateOutputFormat::PrettyJson => {
                let json = serde_json::to_string_pretty(&id).unwrap();
                println!("{}", json);
            }
        }

        Ok(())
    }
}
