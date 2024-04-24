use opendut_carl_api::carl::CarlClient;
use crate::commands::device::{DeviceTable, render_devices};
use crate::ListOutputFormat;

pub async fn execute(carl: &mut CarlClient, output: ListOutputFormat) -> crate::Result<()> {
    let devices = carl.peers.list_devices().await
        .map_err(|_| String::from("Devices could not be listed"))?
        .into_iter()
        .map(DeviceTable::from)
        .collect::<Vec<_>>();

    let text = render_devices(devices, output);
    println!("{text}");
    Ok(())
}