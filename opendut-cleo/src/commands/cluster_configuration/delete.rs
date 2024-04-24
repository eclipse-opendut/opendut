use uuid::Uuid;
use opendut_carl_api::carl::{CarlClient};
use opendut_types::cluster::ClusterId;

pub async fn execute(carl: &mut CarlClient, id: Uuid) -> crate::Result<()> {
    let id = ClusterId::from(id);
    let cluster_configuration = carl.cluster.delete_cluster_configuration(id).await
        .map_err(|error| format!("Failed to delete ClusterConfiguration with id <{id}>.\n  {error}"))?;

    println!("Deleted ClusterConfiguration {} <{}> successfully.", cluster_configuration.name, cluster_configuration.id);

    Ok(())
}
