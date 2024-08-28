use crate::resources::manager::ResourcesManagerRef;
use opendut_carl_api::carl::peer::ListDevicesError;
use opendut_types::peer::PeerDescriptor;
use opendut_types::topology::DeviceDescriptor;
use std::collections::HashMap;
use tracing::{debug, error, info};

pub struct ListDevicesParams {
    pub resources_manager: ResourcesManagerRef,
}

#[tracing::instrument(skip(params), level="trace")]
pub async fn list_devices(params: ListDevicesParams) -> Result<Vec<DeviceDescriptor>, ListDevicesError> {

    async fn inner(params: ListDevicesParams) -> Result<Vec<DeviceDescriptor>, ListDevicesError> {

        let resources_manager = params.resources_manager;

        debug!("Querying all devices.");

        let peers = resources_manager.list::<PeerDescriptor>().await
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
    use crate::actions;
    use crate::actions::peers::testing::{fixture, store_peer_descriptor_options, Fixture};
    use crate::actions::{StorePeerDescriptorOptions, StorePeerDescriptorParams};
    use crate::resources::manager::ResourcesManager;
    use googletest::prelude::*;
    use rstest::rstest;
    use std::sync::Arc;

    #[rstest]
    #[tokio::test]
    async fn should_list_all_devices(fixture: Fixture, store_peer_descriptor_options: StorePeerDescriptorOptions) -> anyhow::Result<()> {
        let resources_manager = ResourcesManager::new_in_memory();

        let result = list_devices(ListDevicesParams {
            resources_manager: Arc::clone(&resources_manager),
        }).await?;
        assert!(result.is_empty());


        actions::store_peer_descriptor(StorePeerDescriptorParams {
            resources_manager: Arc::clone(&resources_manager),
            vpn: fixture.vpn,
            peer_descriptor: fixture.peer_a_descriptor,
            options: store_peer_descriptor_options,
        }).await?;


        let result = list_devices(ListDevicesParams {
            resources_manager: Arc::clone(&resources_manager),
        }).await?;

        let result_ids = result.into_iter()
            .map(|device| device.id)
            .collect::<Vec<_>>();

        assert_that!(
            result_ids,
            unordered_elements_are![
                eq(fixture.peer_a_device_1),
                eq(fixture.peer_a_device_2),
            ]
        );
        Ok(())
    }
}
