use crate::resource::manager::ResourceManagerRef;
use opendut_carl_api::carl::peer::ListPeerDescriptorsError;
use opendut_types::peer::PeerDescriptor;
use tracing::{debug, error, info};
use crate::resource::storage::ResourcesStorageApi;

pub struct ListPeerDescriptorsParams {
    pub resource_manager: ResourceManagerRef,
}

#[tracing::instrument(skip(params), level="trace")]
pub async fn list_peer_descriptors(params: ListPeerDescriptorsParams) -> Result<Vec<PeerDescriptor>, ListPeerDescriptorsError> {

    async fn inner(params: ListPeerDescriptorsParams) -> Result<Vec<PeerDescriptor>, ListPeerDescriptorsError> {

        let resource_manager = params.resource_manager;

        debug!("Querying all peer descriptors.");

        let peers = resource_manager.resources(|resources| {
            resources.list::<PeerDescriptor>()
        }).await
        .map_err(|cause| ListPeerDescriptorsError::Internal { cause: cause.to_string() })?;

        info!("Successfully queried all peer descriptors.");

        Ok(peers)
    }

    inner(params).await
        .inspect_err(|err| error!("{err}"))
}
