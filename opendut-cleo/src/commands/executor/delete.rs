use uuid::Uuid;

use opendut_carl_api::carl::CarlClient;
use opendut_types::peer::executor::{ContainerImage, ExecutorDescriptor};
use opendut_types::peer::PeerId;

pub async fn execute(carl: &mut CarlClient, id: Uuid, images: Vec<ContainerImage>) -> crate::Result<()> {
    let id = PeerId::from(id);

    let mut peer = carl.peers
        .get_peer_descriptor(id)
        .await
        .map_err(|error| format!("Failed to get peer with the id '{}'.\n  {}", id, error))?;

    let container_images = images.into_iter()
        .map(ContainerImage::try_from)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;

    for container_image in container_images {
        peer.executors.executors.retain(|executor| match executor {
            ExecutorDescriptor::Executable => true,
            ExecutorDescriptor::Container { image, .. } => {
                image != &container_image
            }
        } )
    };

    carl.peers.store_peer_descriptor(peer).await
        .map_err(|error| format!("Failed to delete container executor for peer.\n  {}", error))?;

    Ok(())
}
