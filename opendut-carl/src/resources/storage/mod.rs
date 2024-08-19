use url::Url;

use crate::persistence::database::ConnectError;
use crate::persistence::error::PersistenceResult;
use crate::persistence::model::Persistable;
use crate::resources::storage::persistent::PersistentResourcesStorage;
use crate::resources::storage::volatile::VolatileResourcesStorage;
use crate::resources::Resource;

pub mod volatile;
pub mod persistent;

pub enum ResourcesStorage {
    Persistent(PersistentResourcesStorage),
    Volatile(VolatileResourcesStorage),
}
impl ResourcesStorage {
    pub async fn connect(options: PersistenceOptions) -> Result<Self, ConnectionError> {
        let storage = match options {
            PersistenceOptions::Enabled { database_url: url } => {
                let storage = PersistentResourcesStorage::connect(&url).await
                    .map_err(|cause| ConnectionError::Database { url, source: cause })?;
                ResourcesStorage::Persistent(storage)
            }
            PersistenceOptions::Disabled => {
                ResourcesStorage::Volatile(VolatileResourcesStorage::default())
            }
        };
        Ok(storage)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConnectionError {
    #[error("Failed to connect to database at '{url}'")]
    Database { url: Url, #[source] source: ConnectError },
}

#[derive(Clone)]
pub enum PersistenceOptions {
    Enabled {
        database_url: Url,
    },
    Disabled,
}
impl PersistenceOptions {
    pub fn load(config: &config::Config) -> Result<Self, opendut_util::settings::LoadError> {
        let persistence_enabled = config.get_bool("persistence.enabled")?;

        if persistence_enabled {
            let url_field = "persistence.database.url";
            let url_value = config.get_string(url_field)?;
            let database_url = Url::parse(&url_value)
                .map_err(|cause| opendut_util::settings::LoadError::Parse {
                    field: url_field.to_string(),
                    value: url_value,
                    source: Box::new(cause)
                })?;
            Ok(PersistenceOptions::Enabled { database_url })
        } else {
            Ok(PersistenceOptions::Disabled)
        }
    }
}

pub trait ResourcesStorageApi {
    fn insert<R>(&mut self, id: R::Id, resource: R) -> PersistenceResult<()>
    where R: Resource + Persistable;

    fn remove<R>(&mut self, id: R::Id) -> PersistenceResult<Option<R>>
    where R: Resource + Persistable;

    fn get<R>(&self, id: R::Id) -> PersistenceResult<Option<R>>
    where R: Resource + Persistable + Clone;

    fn list<R>(&self) -> PersistenceResult<Vec<R>>
    where R: Resource + Persistable + Clone;
}
