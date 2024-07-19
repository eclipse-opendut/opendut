use std::sync::Mutex;
use std::time::Duration;
use diesel::{Connection as _, ConnectionError, PgConnection};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use tracing::{info, warn};
use url::Url;

use crate::persistence::Db;

pub mod schema;

pub fn connect(url: &Url) -> Result<Db, ConnectError> {
    let connection_retry_interval = Duration::from_secs(5); //TODO move duration into configuration

    let mut connection = loop {
        match PgConnection::establish(url.as_str()) {
            Ok(connection) => break connection,
            Err(cause) => match &cause {
                ConnectionError::BadConnection(_) => {
                    warn!("Connecting to database at {url} failed. Retrying in {interval} ms.", interval=connection_retry_interval.as_millis());
                    std::thread::sleep(connection_retry_interval);
                    continue;
                }
                ConnectionError::CouldntSetupConfiguration(_)
                | ConnectionError::InvalidConnectionUrl(_)
                | ConnectionError::InvalidCString(_) => return Err(ConnectError::Diesel(cause)),
                other => {
                    warn!("Unhandled Diesel ConnectionError variant: {other:?}");
                    return Err(ConnectError::Diesel(cause));
                }
            }
        }
    };
    info!("Connection to database established!");

    run_pending_migrations(&mut connection)
        .map_err(|cause| ConnectError::Migration { source: cause })?;

    Ok(Mutex::new(connection))
}

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("src/persistence/database/migrations/");

fn run_pending_migrations(connection: &mut PgConnection) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let migrated_versions = connection.run_pending_migrations(MIGRATIONS)?;
    let migrated_versions = migrated_versions.into_iter()
        .map(|version| version.to_string())
        .collect::<Vec<String>>()
        .join(", ");
    info!("Completed running pending database migrations: {migrated_versions}");
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum ConnectError {
    #[error("Connection error from Diesel")]
    Diesel(#[source] diesel::ConnectionError),
    #[error("Error while applying migrations")]
    Migration { #[source] source: Box<dyn std::error::Error + Send + Sync> },
}
