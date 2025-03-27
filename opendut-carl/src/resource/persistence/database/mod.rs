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


#[cfg(any(test, doc))] //needed for doctests to compile
pub mod testing {
    use crate::resource::api::global::GlobalResources;
    use crate::resource::manager::{ResourceManager, ResourceManagerRef};
    use crate::resource::storage::{DatabaseConnectInfo, Password, PersistenceOptions};
    use assert_fs::fixture::PathChild;
    use testcontainers_modules::testcontainers::ContainerAsync;
    use testcontainers_modules::{postgres, testcontainers::runners::AsyncRunner};
    use url::Url;

    /// Spawns a Postgres Container and returns a ResourceManager for testing.
    /// ```no_run
    /// # use std::any::Any;
    /// # use opendut_carl::resource::persistence::database;
    ///
    /// #[tokio::test]
    /// async fn test() {
    ///     let mut db = database::testing::spawn_and_connect_resource_manager().await?;
    ///
    ///     do_something_with_resource_manager(db.resource_manager);
    /// }
    ///
    /// # fn do_something_with_resource_manager(resource_manager: impl Any) {}
    /// ```
    pub async fn spawn_and_connect_resource_manager() -> anyhow::Result<PostgresResources> {
        let (container, connect_info, temp_dir) = spawn().await?;

        let global = GlobalResources::default().complete();
        let persistence_options = PersistenceOptions::Enabled {
            database_connect_info: connect_info,
        };
        let resource_manager = ResourceManager::create(global, persistence_options).await?;

        Ok(PostgresResources { resource_manager, container, temp_dir })
    }
    pub struct PostgresResources {
        pub resource_manager: ResourceManagerRef,
        #[allow(unused)] //primarily carried along to extend its lifetime until the end of the test (container is stopped when variable is dropped)
        pub container: ContainerAsync<postgres::Postgres>,
        #[allow(unused)] //carried along to extend its lifetime until the end of the test (database file is deleted when variable is dropped)
        temp_dir: assert_fs::TempDir,
    }

    async fn spawn() -> anyhow::Result<(ContainerAsync<postgres::Postgres>, DatabaseConnectInfo, assert_fs::TempDir)> {
        let temp_dir = assert_fs::TempDir::new()?;
        let file = temp_dir.child("opendut.db");

        let container = postgres::Postgres::default().start().await?;
        let host = container.get_host().await?;
        let port = container.get_host_port_ipv4(5432).await?;

        let connect_info = DatabaseConnectInfo {
            file: file.to_path_buf(),
            url: Url::parse(&format!("postgres://{host}:{port}/postgres"))?,
            username: String::from("postgres"),
            password: Password::new_static("postgres"),
        };

        Ok((container, connect_info, temp_dir))
    }
}
