use crate::resource::api::resources::Resources;
use crate::resource::storage::ResourcesStorageApi;
use opendut_model::peer::PeerDescriptor;
use opendut_model::topology::DeviceDescriptor;
use std::collections::HashMap;
use tracing::{debug, info};
use crate::resource::persistence::error::PersistenceError;

impl Resources<'_> {
    #[tracing::instrument(skip_all, level="trace")]
    pub fn list_devices(&self) -> Result<Vec<DeviceDescriptor>, ListDevicesError> {

        debug!("Querying all devices.");

        let peers = self.list::<PeerDescriptor>()
            .map_err(|source| ListDevicesError::Persistence { source })?;

        let devices = peers.into_iter()
            .flat_map(|(_, peer_descriptor) | peer_descriptor.topology.devices)
            .map(|device| (device.id, device))
            .collect::<HashMap<_, _>>();

        let devices = devices.into_values().collect::<Vec<_>>();

        info!("Successfully queried all peers.");

        Ok(devices)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ListDevicesError {
    #[error("Error when accessing persistence while listing devices")]
    Persistence {
        #[source] source: PersistenceError,
    }
}

#[cfg(test)]
mod tests {
    use crate::manager::peer_manager::StorePeerDescriptorParams;
    use crate::manager::testing::PeerFixture;
    use crate::resource::manager::ResourceManager;
    use crate::settings::vpn::Vpn;
    use googletest::prelude::*;

    #[tokio::test]
    async fn should_list_all_devices() -> anyhow::Result<()> {
        let peer = PeerFixture::new();

        let resource_manager = ResourceManager::new_in_memory();

        let result = resource_manager.resources(async |resources|
            resources.list_devices()
        ).await??;
        assert!(result.is_empty());


        resource_manager.resources_mut(async |resources| {
            resources.store_peer_descriptor(StorePeerDescriptorParams {
                vpn: Vpn::Disabled,
                peer_descriptor: peer.descriptor,
            }).await
        }).await??;


        let result = resource_manager.resources(async |resources|
            resources.list_devices()
        ).await??;

        let result_ids = result.into_iter()
            .map(|device| device.id)
            .collect::<Vec<_>>();

        assert_that!(
            result_ids,
            unordered_elements_are![
                eq(&peer.device_1),
                eq(&peer.device_2),
            ]
        );
        Ok(())
    }
}
