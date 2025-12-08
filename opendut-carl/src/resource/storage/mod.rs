use crate::resource::api::id::ResourceId;
use crate::resource::api::resources::RelayedSubscriptionEvents;
use crate::resource::api::Resource;
use crate::resource::persistence::error::PersistenceResult;
use crate::resource::persistence::persistable::{Persistable, StorageKind};
use crate::resource::persistence::{Db, Memory};
use crate::resource::subscription::Subscribable;
use crate::resource::{persistence, ConnectError};
use anyhow::anyhow;
use prost::Message;
use redb::backends::InMemoryBackend;
use std::collections::HashMap;
use std::fmt::Display;
use std::fs;
use std::ops::Not;
use std::path::PathBuf;
use redb::ReadableDatabase;
use tracing::{debug, info};

#[cfg(test)]
mod tests;

pub struct ResourceStorage {
    db: redb::Database,
    memory: redb::Database,
}
impl ResourceStorage {
    pub async fn connect(options: &PersistenceOptions) -> Result<Self, ConnectError> {

        let db = match options {
            PersistenceOptions::Enabled { database_connect_info } => {
                let file = &database_connect_info.file;

                if let Some(parent_dir) = file.parent() {
                    fs::create_dir_all(parent_dir)
                        .map_err(|source| ConnectError::DatabaseDirCreate { dir: parent_dir.to_owned(), source })?;
                }

                if file.exists().not() {
                    info!("Database file at {file:?} does not exist. Creating an empty database.");
                }

                let db = redb::Database::create(file)
                    .map_err(|source| ConnectError::DatabaseCreate { file: file.to_owned(), source })?;
                debug!("Database file opened from: {file:?}");
                db
            }
            PersistenceOptions::Disabled => {
                let db = redb::Database::builder()
                    .create_with_backend(InMemoryBackend::new())
                    .map_err(ConnectError::DatabaseInMemoryCreate)?;
                debug!("Database opened in-memory.");
                db
            }
        };

        let memory = redb::Database::builder()
            .create_with_backend(InMemoryBackend::new())
            .map_err(ConnectError::DatabaseInMemoryCreate)?;

        Ok(Self { db, memory })
    }

    pub async fn resources<T, F>(&self, code: F) -> PersistenceResult<T>
    where
        F: AsyncFnOnce(ResourceTransaction) -> T,
    {
        let mut relayed_subscription_events = RelayedSubscriptionEvents::default();

        let db_transaction = self.db.begin_read()?;
        let memory_transaction = self.memory.begin_read()?;
        let result = {
            let transaction = ResourceTransaction {
                db: Db::Read(&db_transaction),
                memory: Memory::Read(&memory_transaction),
                relayed_subscription_events: &mut relayed_subscription_events,
            };

            code(transaction).await
        };

        debug_assert!(relayed_subscription_events.is_empty(), "Read-only storage operations should not trigger any subscription events.");

        Ok(result)
    }

    pub async fn resources_mut<T, E, F>(&mut self, code: F) -> PersistenceResult<(Result<T, E>, RelayedSubscriptionEvents)>
    where
        F: AsyncFnOnce(ResourceTransaction) -> Result<T, E>,
        E: Display,
    {
        let mut relayed_subscription_events = RelayedSubscriptionEvents::default();
        let mut db_transaction = self.db.begin_write()?;
        let mut memory_transaction = self.memory.begin_write()?;
        let result = {
            let persistent_transaction = ResourceTransaction {
                db: Db::ReadWrite(&mut db_transaction),
                memory: Memory::ReadWrite(&mut memory_transaction),
                relayed_subscription_events: &mut relayed_subscription_events,
            };

            code(persistent_transaction).await
        };

        match &result {
            Ok(_) => {
                db_transaction.commit()?;
                memory_transaction.commit()?;
            }
            Err(cause) => {
                debug!("Not committing changes to the database due to error:\n  {cause}");
            }
        }

        Ok((result, relayed_subscription_events))
    }
}

pub struct ResourceTransaction<'transaction> {
    db: Db<'transaction>,
    memory: Memory<'transaction>,
    pub relayed_subscription_events: &'transaction mut RelayedSubscriptionEvents,
}
impl ResourcesStorageApi for ResourceTransaction<'_> {
    fn insert<R>(&mut self, id: R::Id, resource: R) -> PersistenceResult<()>
    where R: Resource + Persistable {
        let db = self.get_db_for_resource::<R>();

        let key = persistence::Key::from(ResourceId::<R>::into_id(id));

        let value = R::Proto::from(resource).encode_to_vec();

        let mut table = db.read_write_table(R::TABLE_DEFINITION)?;
        table.insert(key, value)?;

        Ok(())
    }

    fn remove<R>(&mut self, id: R::Id) -> PersistenceResult<Option<R>>
    where R: Resource + Persistable {
        let db = self.get_db_for_resource::<R>();

        let key = persistence::Key::from(ResourceId::<R>::into_id(id));

        let mut table = db.read_write_table(R::TABLE_DEFINITION)?;

        let value = table.remove(key)?
            .map(|value| R::try_from_bytes(value.value()))
            .transpose()?;

        Ok(value)
    }

    fn get<R>(&self, id: R::Id) -> PersistenceResult<Option<R>>
    where
        R: Resource + Persistable + Clone
    {
        let db = self.get_db_for_resource::<R>();

        let key = persistence::Key::from(ResourceId::<R>::into_id(id));

        if let Some(table) = db.read_table(R::TABLE_DEFINITION)? {
            let value = table.get(&key)?
                .map(|value| R::try_from_bytes(value.value()))
                .transpose()?;
            Ok(value)
        } else {
            Ok(None)
        }
    }

    fn list<R>(&self) -> PersistenceResult<HashMap<R::Id, R>>
    where
        R: Resource + Persistable + Clone
    {
        let db = self.get_db_for_resource::<R>();

        if let Some(table) = db.read_table(R::TABLE_DEFINITION)? {
            table.iter()?
                .map(|value| {
                    let (key, value) = value?;
                    let id = ResourceId::<R>::from_id(key.value().id);
                    let value = R::try_from_bytes(value.value())?;

                    Ok((id, value))
                })
                .collect::<PersistenceResult<HashMap<_, _>>>()
        } else {
            Ok(HashMap::default())
        }
    }
}
impl ResourceTransaction<'_> {
    fn get_db_for_resource<R: Persistable>(&self) -> &Db<'_> {
        match R::STORAGE {
            StorageKind::Persistent => &self.db,
            StorageKind::Volatile => &self.memory,
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

                let value = expand_tilde(value);
                let path = PathBuf::from(&value);
                if path.is_relative() {
                    return Err(LoadError::ParseValue { field, value, source: anyhow!("Path to the database file should be absolute!").into() });
                }
                path
            };

            Ok(PersistenceOptions::Enabled {
                database_connect_info: DatabaseConnectInfo {
                    file,
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


fn expand_tilde(path: String) -> String {
    if path.contains('~') {
        let home_dir = std::env::home_dir()
            .expect("Could not determine home directory of user. Needed for expanding '~' in path.");
        let home_dir = home_dir.to_str()
            .expect("Could not convert home directory path to string.");

        path.replace('~', home_dir)
    } else {
        path
    }
}

#[cfg(test)]
mod expand_tilde_tests {
    use super::expand_tilde;

    #[test]
    fn should_expand_tilde() {
        let home_dir = std::env::home_dir().unwrap();
        let home_dir = home_dir.to_string_lossy();

        let testee = String::from("~");
        assert_eq!(expand_tilde(testee), home_dir);

        let testee = String::from("~/test");
        assert_eq!(expand_tilde(testee), format!("{home_dir}/test"));
    }
}
