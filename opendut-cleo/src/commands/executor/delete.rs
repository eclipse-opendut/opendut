use uuid::Uuid;

use opendut_carl_api::carl::CarlClient;
use opendut_types::peer::executor::{container::ContainerImage, ExecutorKind};
use opendut_types::peer::PeerId;

/// Delete a container executor
#[derive(clap::Parser)]
pub struct DeleteContainerExecutorCli {
    ///ID of the peer to delete the container executor from
    #[arg()]
    peer_id: Uuid,
    ///Container images to delete
    #[arg(short, long, num_args = 1.., required = true)]
    images: Vec<ContainerImage>,
}

impl DeleteContainerExecutorCli {
    pub async fn execute(self, carl: &mut CarlClient) -> crate::Result<()> {
        let id = PeerId::from(self.peer_id);

        let mut peer = carl.peers
            .get_peer_descriptor(id)
            .await
            .map_err(|error| format!("Failed to get peer with the id '{}'.\n  {}", id, error))?;

        let container_images = self.images.into_iter()
            .map(ContainerImage::try_from)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| error.to_string())?;

        for container_image in container_images {
            peer.executors.executors.retain(|executor| match &executor.kind {
                ExecutorKind::Executable => true,
                ExecutorKind::Container { image, .. } => {
                    image != &container_image
                }
            })
        };

        carl.peers.store_peer_descriptor(peer).await
            .map_err(|error| format!("Failed to delete container executor for peer.\n  {}", error))?;

        Ok(())
    }
}