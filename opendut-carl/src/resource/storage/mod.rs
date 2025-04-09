use crate::resource::api::global::GlobalResourcesRef;
use crate::resource::api::resources::{RelayedSubscriptionEvents, Resources};
use crate::resource::api::Resource;
use crate::resource::ConnectError;
use crate::resource::persistence::error::PersistenceResult;
use crate::resource::persistence::resources::Persistable;
use crate::resource::storage::persistent::PersistentResourcesStorage;
use crate::resource::storage::volatile::VolatileResourcesStorageHandle;
use crate::resource::subscription::Subscribable;
use anyhow::anyhow;
use std::collections::HashMap;
use std::fmt::Display;
use std::path::PathBuf;

pub mod volatile;
pub mod persistent;

#[cfg(test)]
mod tests;

pub enum ResourceStorage {
    Persistent(PersistentResourcesStorage),
    Volatile(VolatileResourcesStorageHandle),
}
impl ResourceStorage {
    pub async fn connect(options: &PersistenceOptions) -> Result<Self, ConnectError> {
        let storage = match options {
            PersistenceOptions::Enabled { database_connect_info } => {
                let storage = PersistentResourcesStorage::connect(database_connect_info).await?;
                ResourceStorage::Persistent(storage)
            }
            PersistenceOptions::Disabled => {
                ResourceStorage::Volatile(VolatileResourcesStorageHandle::default())
            }
        };
        Ok(storage)
    }

    pub(super) async fn resources<T, F>(&self, global: GlobalResourcesRef, code: F) -> PersistenceResult<T>
    where
        F: AsyncFnOnce(&mut Resources) -> T,
    {
        match self {
            ResourceStorage::Persistent(storage) => storage.resources(async |transaction| {
                let mut transaction = Resources::persistent(transaction, global);
                code(&mut transaction).await
            }).await,
            ResourceStorage::Volatile(storage) => Ok(
                storage.resources(async |transaction| {
                    let mut transaction = Resources::volatile(transaction, global);
                    code(&mut transaction).await
                }).await
            ),
        }
    }

    pub(super) async fn resources_mut<T, E, F>(&mut self, global: GlobalResourcesRef, code: F) -> PersistenceResult<(Result<T, E>, RelayedSubscriptionEvents)>
    where
        F: AsyncFnOnce(&mut Resources) -> Result<T, E>,
        E: Display,
    {
        match self {
            ResourceStorage::Persistent(storage) => storage.resources_mut(async |transaction| {
                let mut transaction = Resources::persistent(transaction, global);
                code(&mut transaction).await
            }).await,
            ResourceStorage::Volatile(storage) => storage.resources_mut(async |transaction| {
                let mut transaction = Resources::volatile(transaction, global);
                code(&mut transaction).await
            }).await,
        }
    }
}

#[cfg(test)]
impl ResourceStorage {
    pub async fn contains<R>(&self, id: R::Id) -> bool
    where R: Resource {
        match self {
            ResourceStorage::Persistent(_) => unimplemented!(),
            ResourceStorage::Volatile(storage) => storage.contains::<R>(id),
        }
    }

    pub async fn is_empty(&self) -> bool {
        match self {
            ResourceStorage::Persistent(_) => unimplemented!(),
            ResourceStorage::Volatile(storage) => storage.is_empty(),
        }
    }
}


pub enum PersistenceOptions {
    Enabled { database_connect_info: DatabaseConnectInfo },
    Disabled,
}
impl PersistenceOptions {
    pub fn load(config: &config::Config) -> Result<Self, opendut_util::settings::LoadError> {
        use opendut_util::settings::LoadError;

        let persistence_enabled = config.get_bool("persistence.enabled")?;

        if persistence_enabled {
            let file = {
                let field = "persistence.database.file";
                let value = config.get_string(field)
                    .map_err(|cause| LoadError::ReadField { field, source: Box::new(cause) })?;

                if value.is_empty() {
                    return Err(LoadError::ParseValue { field, value, source: anyhow!("Path to the database file has to be specified!").into() });
                }

                let path = PathBuf::from(&value);
                if path.is_relative() {
                    return Err(LoadError::ParseValue { field, value, source: anyhow!("Path to the database file should be absolute!").into() });
                }
                path
            };

            #[cfg(feature="postgres")]
            let url = {
                let field = "persistence.database.url";
                let value = config.get_string(field)
                    .map_err(|cause| LoadError::ReadField { field, source: Box::new(cause) })?;

                url::Url::parse(&value)
                    .map_err(|cause| LoadError::ParseValue { field, value, source: Box::new(cause) })?
            };

            #[cfg(feature="postgres")]
            let username = {
                let field = "persistence.database.username";
                config.get_string(field)
                    .map_err(|source| LoadError::ReadField { field, source: Box::new(source) })?
            };

            #[cfg(feature="postgres")]
            let password = {
                let field = "persistence.database.password";
                let value = config.get_string(field)
                    .map_err(|source| LoadError::ReadField { field, source: Box::new(source) })?;
                Password { secret: value }
            };

            Ok(PersistenceOptions::Enabled {
                database_connect_info: DatabaseConnectInfo {
                    file,

                    #[cfg(feature="postgres")]
                    url,
                    #[cfg(feature="postgres")]
                    username,
                    #[cfg(feature="postgres")]
                    password,
                }
            })
        } else {
            Ok(PersistenceOptions::Disabled)
        }
    }
}
#[derive(Clone)]
pub struct DatabaseConnectInfo {
    pub file: PathBuf,


    #[cfg(feature="postgres")]
    /// Deprecated
    pub url: url::Url,

    #[cfg(feature="postgres")]
    /// Deprecated
    pub username: String,

    #[cfg(feature="postgres")]
    /// Deprecated
    pub password: Password,
}

#[cfg(feature="postgres")]
///Wrapper for String without Debug and Display
#[derive(Clone)]
pub struct Password { secret: String }

#[cfg(feature="postgres")]
impl Password {
    pub fn secret(&self) -> &str {
        &self.secret
    }

    #[cfg(test)]
    pub fn new_static(secret: &'static str) -> Self {
        Self { secret: secret.to_owned() }
    }
}

pub trait ResourcesStorageApi {
    fn insert<R>(&mut self, id: R::Id, resource: R) -> PersistenceResult<()>
    where R: Resource + Persistable + Subscribable;

    fn remove<R>(&mut self, id: R::Id) -> PersistenceResult<Option<R>>
    where R: Resource + Persistable + Subscribable;

    fn get<R>(&self, id: R::Id) -> PersistenceResult<Option<R>>
    where R: Resource + Persistable + Clone;

    fn list<R>(&self) -> PersistenceResult<HashMap<R::Id, R>>
    where R: Resource + Persistable + Clone;
}
