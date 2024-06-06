use opendut_carl_api::carl::CarlClient;
use crate::commands::accessory::{AccessoryTable, render_accessories};
use crate::ListOutputFormat;

/// List all accessories
#[derive(clap::Parser)]
pub struct ListAccessoriesCli;

impl ListAccessoriesCli {
    pub async fn execute(self, carl: &mut CarlClient, output: ListOutputFormat) -> crate::Result<()> {
        let accessories = carl.peers.list_accessories().await
            .map_err(|_| String::from("Accessories could not be listed"))?
            .into_iter()
            .map(AccessoryTable::from)
            .collect::<Vec<_>>();

        let text = render_accessories(accessories, output);
        println!("{text}");
        Ok(())
    }
}