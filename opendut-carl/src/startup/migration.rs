#![allow(deprecated)]

use tracing::{debug, info};
use crate::resource::manager::ResourceManagerRef;


pub async fn run(resource_manager: ResourceManagerRef) -> anyhow::Result<()> {
    debug!("Running migrations...");

    perform_database_upgrade(resource_manager).await?;

    debug!("Migrations complete.");
    Ok(())
}

async fn perform_database_upgrade(resource_manager: ResourceManagerRef) -> anyhow::Result<()> {
    debug!("Performing database upgrade, if necessary...");

    let performed_upgrade = resource_manager.perform_database_upgrade().await?;

    if performed_upgrade {
        info!("Completed database upgrade.");
    } else {
        debug!("No database upgrade had to be performed.");
    };

    Ok(())
}
