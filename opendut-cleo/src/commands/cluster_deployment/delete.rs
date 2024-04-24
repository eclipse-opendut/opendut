use uuid::Uuid;
use opendut_carl_api::carl::{CarlClient};
use opendut_types::cluster::ClusterId;

pub async fn execute(carl: &mut CarlClient, id: Uuid) -> crate::Result<()> {
    let id = ClusterId::from(id);
    carl.cluster.delete_cluster_deployment(id).await
        .map_err(|error| {
            format!("Could not delete cluster deployment for ClusterID '{}'.\n  {}", id, error)
        })?;
    println!("Deleted cluster deployment for ClusterID '{}'.", id);

    Ok(())
}
