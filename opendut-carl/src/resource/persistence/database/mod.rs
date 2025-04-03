use crate::resource::storage::DatabaseConnectInfo;
use crate::resource::ConnectError;
use backon::Retryable;
use diesel::{Connection as _, ConnectionError, PgConnection};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use tracing::{debug, info, warn};

pub mod schema;

pub async fn connect(database_connect_info: &DatabaseConnectInfo) -> Result<PgConnection, ConnectError> {
    let DatabaseConnectInfo { url, username, password, .. } = database_connect_info;

    let confidential_url = {
        let mut url = url.clone();
        url.set_username(username)
            .expect("failed to set username on URL while connecting to database");
        url.set_password(Some(password.secret()))
            .expect("failed to set password on URL while connecting to database");
        url
    };

    let mut connection = (|| async {
        PgConnection::establish(confidential_url.as_str())
    })
        .retry(backon::ExponentialBuilder::default())
        .when(|cause| match &cause {
            ConnectionError::BadConnection(_) => {
                true
            }
            ConnectionError::CouldntSetupConfiguration(_)
            | ConnectionError::InvalidConnectionUrl(_)
            | ConnectionError::InvalidCString(_) => {
                false
            }
            other => {
                warn!("Unhandled Diesel ConnectionError variant: {other:?}");
                false
            }
        })
        .notify(|cause, after| {
            warn!("Connecting to database at {url} failed. Retrying in {after:?}.\n  {cause}");
        })
        .await
        .map_err(|source| ConnectError::Diesel { url: url.to_owned(), source })?;

    info!("Connection to database at {url} established!");

    run_pending_migrations(&mut connection)
        .map_err(|cause| ConnectError::Migration { source: cause })?;

    Ok(connection)
}

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("src/resource/persistence/database/migrations/");

fn run_pending_migrations(connection: &mut PgConnection) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let migrated_versions = connection.run_pending_migrations(MIGRATIONS)?;

    if migrated_versions.is_empty() {
        debug!("No database migrations had to be applied.");
    } else {
        let migrated_versions = migrated_versions.into_iter()
            .map(|version| version.to_string())
            .collect::<Vec<String>>()
            .join(", ");
        info!("Completed running pending database migrations: {migrated_versions}");
    }
    Ok(())
}
