use std::sync::{Arc, Mutex};
use diesel::{Connection as _, PgConnection};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use tracing::log;
use url::Url;

pub mod schema;

pub type Db = Arc<Mutex<PgConnection>>; //Mutex rather than RwLock, because we share this between threads (i.e. we need it to implement `Sync`)

pub fn connect(url: &Url) -> Result<Db, ConnectError> {
    let mut connection = PgConnection::establish(url.as_str())?;
    log::info!("Connection to database established!");

    run_pending_migrations(&mut connection)
        .map_err(|cause| ConnectError::Migration { source: cause })?;

    Ok(Arc::new(Mutex::new(connection)))
}

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("src/persistence/database/migrations/");

fn run_pending_migrations(connection: &mut PgConnection) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let migrated_versions = connection.run_pending_migrations(MIGRATIONS)?;
    log::info!("Completed running pending database migrations: {migrated_versions:#?}");
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum ConnectError {
    #[error("Connection error from Diesel")]
    Diesel(#[from] diesel::ConnectionError),
    #[error("Error while applying migrations")]
    Migration { #[source] source: Box<dyn std::error::Error + Send + Sync> },
}
