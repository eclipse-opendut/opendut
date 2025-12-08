#![allow(deprecated)]

use tracing::debug;
use crate::resource::manager::ResourceManagerRef;


pub async fn run(_resource_manager: ResourceManagerRef) -> anyhow::Result<()> {
    debug!("Running migrations...");

    //insert migrations here

    debug!("Migrations complete.");
    Ok(())
}
