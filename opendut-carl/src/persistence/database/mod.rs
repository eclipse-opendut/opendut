use backon::Retryable;
use diesel::{Connection as _, ConnectionError, PgConnection};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use tracing::{debug, info, warn};
use crate::resources::storage::DatabaseConnectInfo;

pub mod schema;

pub async fn connect(database_connect_info: &DatabaseConnectInfo) -> Result<PgConnection, ConnectError> {
    let DatabaseConnectInfo { url, username, password } = database_connect_info;

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
        .map_err(ConnectError::Diesel)?;

    info!("Connection to database at {url} established!");

    run_pending_migrations(&mut connection)
        .map_err(|cause| ConnectError::Migration { source: cause })?;

    Ok(connection)
}

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("src/persistence/database/migrations/");

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

#[derive(Debug, thiserror::Error)]
pub enum ConnectError {
    #[error("Connection error from Diesel")]
    Diesel(#[source] diesel::ConnectionError),
    #[error("Error while applying migrations")]
    Migration { #[source] source: Box<dyn std::error::Error + Send + Sync> },
}


#[cfg(any(test, doc))] //needed for doctests to compile
pub mod testing {
    use crate::persistence::database;
    use diesel::{Connection, PgConnection};
    use testcontainers_modules::testcontainers::ContainerAsync;
    use testcontainers_modules::{postgres, testcontainers::runners::AsyncRunner};
    use url::Url;
    use crate::resources::manager::{ResourcesManager, ResourcesManagerRef};
    use crate::resources::storage::{DatabaseConnectInfo, Password, PersistenceOptions};

    /// Spawns a Postgres Container and returns a connection for testing.
    /// ```no_run
    /// # use diesel::PgConnection;
    /// # use opendut_carl::persistence::database;
    ///
    /// #[tokio::test]
    /// async fn test() {
    ///     let mut db = database::testing::spawn_and_connect().await?;
    ///
    ///     do_something_with_database(db.connection);
    /// }
    ///
    /// # fn do_something_with_database(connection: PgConnection) {}
    /// ```
    pub async fn spawn_and_connect() -> anyhow::Result<PostgresConnection> {
        let (container, connect_info) = spawn().await?;

        let mut connection = database::connect(&connect_info).await?;
        connection.begin_test_transaction()?;
        Ok(PostgresConnection { container, connection })
    }
    pub struct PostgresConnection {
        #[allow(unused)] //primarily carried along to extend its lifetime until the end of the test (container is stopped when variable is dropped)
        pub container: ContainerAsync<postgres::Postgres>,
        pub connection: PgConnection,
    }

    /// Spawns a Postgres Container and returns a ResourcesManager for testing.
    /// ```no_run
    /// # use std::any::Any;
    /// # use opendut_carl::persistence::database;
    ///
    /// #[tokio::test]
    /// async fn test() {
    ///     let mut db = database::testing::spawn_and_connect_resources_manager().await?;
    ///
    ///     do_something_with_resources_manager(db.resources_manager);
    /// }
    ///
    /// # fn do_something_with_resources_manager(resources_manager: impl Any) {}
    /// ```
    pub async fn spawn_and_connect_resources_manager() -> anyhow::Result<PostgresResources> {
        let (container, connect_info) = spawn().await?;

        let resources_manager = ResourcesManager::create(PersistenceOptions::Enabled {
            database_connect_info: connect_info.clone(),
        }).await?;

        Ok(PostgresResources { container, resources_manager })
    }
    pub struct PostgresResources {
        #[allow(unused)] //primarily carried along to extend its lifetime until the end of the test (container is stopped when variable is dropped)
        pub container: ContainerAsync<postgres::Postgres>,
        pub resources_manager: ResourcesManagerRef,
    }

    async fn spawn() -> anyhow::Result<(ContainerAsync<postgres::Postgres>, DatabaseConnectInfo)> {
        let container = postgres::Postgres::default().start().await?;
        let host = container.get_host().await?;
        let port = container.get_host_port_ipv4(5432).await?;

        let connect_info = DatabaseConnectInfo {
            url: Url::parse(&format!("postgres://{host}:{port}/postgres"))?,
            username: String::from("postgres"),
            password: Password::new_static("postgres"),
        };

        Ok((container, connect_info))
    }
}
