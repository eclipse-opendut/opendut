use uuid::Uuid;
use opendut_carl_api::carl::CarlClient;
use opendut_types::topology::DeviceId;

pub async fn execute(carl: &mut CarlClient, device_id: Uuid) -> crate::Result<()> {
    let device_id = DeviceId::from(device_id);

    let mut peers = carl.peers.list_peer_descriptors().await
        .map_err(|error| format!("Could not list peers.\n  {}", error))?;

    let peer = peers.iter_mut().find(|peer| {
        peer.topology.devices
            .iter()
            .any(|device| device.id == device_id)
    }).ok_or(format!("Cannot find a peer with the device <{}>.", device_id))?;

    peer.topology.devices.retain(|device| device.id != device_id);

    carl.peers.store_peer_descriptor(Clone::clone(peer)).await
        .map_err(|error| format!("Failed to delete peer.\n  {}", error))?;

    Ok(())
}
