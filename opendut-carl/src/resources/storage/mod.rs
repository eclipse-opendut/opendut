use url::Url;

use opendut_types::resources::Id;

use crate::persistence::database::ConnectError;
use crate::resources::{Iter, IterMut, Resource, Update};
use crate::resources::storage::persistent::PersistentResourcesStorage;
use crate::resources::storage::volatile::VolatileResourcesStorage;

pub mod volatile;
pub mod persistent;

pub enum ResourcesStorage {
    Persistent(PersistentResourcesStorage),
    Volatile(VolatileResourcesStorage),
}
impl ResourcesStorage {
    pub fn connect(options: PersistenceOptions) -> Result<Self, ConnectionError> {
        let storage = match options {
            PersistenceOptions::Enabled { database_url: url } => {
                let storage = PersistentResourcesStorage::connect(&url)
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
    fn insert<R>(&mut self, id: Id, resource: R)
    where R: Resource;

    fn update<R>(&mut self, id: Id) -> Update<R>
    where R: Resource;

    fn remove<R>(&mut self, id: Id) -> Option<R>
    where R: Resource;

    fn get<R>(&self, id: Id) -> Option<R>
    where R: Resource + Clone;

    fn iter<R>(&self) -> Iter<R>
    where R: Resource;

    fn iter_mut<R>(&mut self) -> IterMut<R>
    where R: Resource;
}
