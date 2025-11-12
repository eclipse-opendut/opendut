#![allow(deprecated)]

use tracing::debug;
use crate::resource::manager::ResourceManagerRef;


pub async fn run(_resource_manager: ResourceManagerRef) -> anyhow::Result<()> {
    debug!("Running migrations...");

    debug!("No migrations need to be applied."); //replace with proper migrations, if you have any

    debug!("Migrations complete.");
    Ok(())
}
