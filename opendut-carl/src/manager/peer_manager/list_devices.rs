use crate::resource::manager::ResourceManagerRef;
use opendut_carl_api::carl::peer::ListDevicesError;
use opendut_types::peer::PeerDescriptor;
use opendut_types::topology::DeviceDescriptor;
use std::collections::HashMap;
use tracing::{debug, error, info};

pub struct ListDevicesParams {
    pub resource_manager: ResourceManagerRef,
}

#[tracing::instrument(skip(params), level="trace")]
pub async fn list_devices(params: ListDevicesParams) -> Result<Vec<DeviceDescriptor>, ListDevicesError> {

    async fn inner(params: ListDevicesParams) -> Result<Vec<DeviceDescriptor>, ListDevicesError> {

        let resource_manager = params.resource_manager;

        debug!("Querying all devices.");

        let peers = resource_manager.list::<PeerDescriptor>().await
            .map_err(|cause| ListDevicesError::Internal { cause: cause.to_string() })?;

        let devices = peers.into_iter()
            .flat_map(|peer| peer.topology.devices)
            .map(|device| (device.id, device))
            .collect::<HashMap<_, _>>();

        let devices = devices.into_values().collect::<Vec<_>>();

        info!("Successfully queried all peers.");

        Ok(devices)
    }

    inner(params).await
        .inspect_err(|err| error!("{err}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manager::testing::PeerFixture;
    use crate::resource::manager::ResourceManager;
    use crate::settings::vpn::Vpn;
    use googletest::prelude::*;
    use std::sync::Arc;
    use crate::manager::peer_manager;
    use crate::manager::peer_manager::StorePeerDescriptorParams;

    #[tokio::test]
    async fn should_list_all_devices() -> anyhow::Result<()> {
        let peer = PeerFixture::new();

        let resource_manager = ResourceManager::new_in_memory();

        let result = list_devices(ListDevicesParams {
            resource_manager: Arc::clone(&resource_manager),
        }).await?;
        assert!(result.is_empty());


        peer_manager::store_peer_descriptor(StorePeerDescriptorParams {
            resource_manager: Arc::clone(&resource_manager),
            vpn: Vpn::Disabled,
            peer_descriptor: peer.descriptor,
        }).await?;


        let result = list_devices(ListDevicesParams {
            resource_manager: Arc::clone(&resource_manager),
        }).await?;

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
