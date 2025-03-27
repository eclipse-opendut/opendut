use crate::resource::api::resources::RelayedSubscriptionEvents;
use crate::resource::persistence::error::PersistenceResult;
use crate::resource::persistence::resources::Persistable;
use crate::resource::persistence::Storage;
use crate::resource::storage::volatile::VolatileResourcesStorage;
use crate::resource::storage::{DatabaseConnectInfo, Resource, ResourcesStorageApi};
use crate::resource::ConnectError;
use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::fs;
use std::sync::{Arc, Mutex};
use tracing::debug;

pub struct PersistentResourcesStorage {
    db: redb::Database,
    memory: Arc<Mutex<VolatileResourcesStorage>>,
}
impl PersistentResourcesStorage {
    pub async fn connect(database_connect_info: &DatabaseConnectInfo) -> Result<Self, ConnectError> {
        let _ = crate::resource::persistence::database::connect(database_connect_info).await?; //TODO remove or use for migration

        let file = &database_connect_info.file;

        if let Some(parent_dir) = file.parent() {
            fs::create_dir_all(parent_dir)
                .map_err(|source| ConnectError::DatabaseDirCreate { dir: parent_dir.to_owned(), source })?;
        }

        let db = redb::Database::create(file)
            .map_err(|source| ConnectError::DatabaseCreate { file: file.to_owned(), source })?;
        debug!("Database file opened from: {file:?}");

        let memory = VolatileResourcesStorage::default();
        let memory = Arc::new(Mutex::new(memory));

        Ok(Self { db, memory })
    }

    pub async fn resources<T, F>(&self, code: F) -> T
    where
        F: AsyncFnOnce(PersistentResourcesTransaction) -> T,
    {
        let mut relayed_subscription_events = RelayedSubscriptionEvents::default();

        let mut transaction = self.db.begin_write().unwrap(); //TODO don't unwrap //TODO don't begin_write(), but rather begin_read() ?
        let result = {
            let transaction = PersistentResourcesTransaction {
                db: Mutex::new(&mut transaction), //TODO don't unwrap
                memory: self.memory.clone(),
                relayed_subscription_events: &mut relayed_subscription_events,
            };

            code(transaction).await
        };
        transaction.commit().unwrap(); //TODO don't unwrap

        debug_assert!(relayed_subscription_events.is_empty(), "Read-only storage operations should not trigger any subscription events.");

        result
    }

    pub async fn resources_mut<T, E, F>(&mut self, code: F) -> PersistenceResult<(Result<T, E>, RelayedSubscriptionEvents)>
    where
        F: AsyncFnOnce(PersistentResourcesTransaction) -> Result<T, E>,
        E: Display + Send + Sync + 'static,
    {
        let mut relayed_subscription_events = RelayedSubscriptionEvents::default();
        let mut transaction = self.db.begin_write().unwrap(); //TODO don't unwrap
        let result = {
            let persistent_transaction = PersistentResourcesTransaction {
                db: Mutex::new(&mut transaction),
                memory: self.memory.clone(),
                relayed_subscription_events: &mut relayed_subscription_events,
            };

            code(persistent_transaction).await
        };

        match &result {
            Ok(_) => {
                transaction.commit().unwrap(); //TODO don't unwrap
            }
            Err(cause) => {
                debug!("Not committing changes to the database due to error:\n  {cause}");
            }
        }

        Ok((result, relayed_subscription_events))
    }
}

impl ResourcesStorageApi for PersistentResourcesStorage {
    fn insert<R>(&mut self, id: R::Id, resource: R) -> PersistenceResult<()>
    where R: Resource + Persistable {
        let mut transaction = self.db.begin_write().unwrap(); //TODO don't unwrap
        let mut storage = Storage {
            db: &mut transaction,
            memory: self.memory.clone(),
        };
        let result = resource.insert(id, &mut storage);
        transaction.commit().unwrap(); //TODO don't unwrap
        result
    }

    fn remove<R>(&mut self, id: R::Id) -> PersistenceResult<Option<R>>
    where R: Resource + Persistable {
        let mut transaction = self.db.begin_write().unwrap(); //TODO don't unwrap
        let mut storage = Storage {
            db: &mut transaction,
            memory: self.memory.clone(),
        };
        let result = R::remove(id, &mut storage);
        transaction.commit().unwrap(); //TODO don't unwrap
        result
    }

    fn get<R>(&self, id: R::Id) -> PersistenceResult<Option<R>>
    where R: Resource + Persistable + Clone {
        let storage = Storage {
            db: &mut self.db.begin_write().unwrap(), //TODO don't unwrap //TODO begin_read()
            memory: self.memory.clone(),
        };
        R::get(id, &storage)
    }

    fn list<R>(&self) -> PersistenceResult<HashMap<R::Id, R>>
    where R: Resource + Persistable + Clone {
        let storage = Storage {
            db: &mut self.db.begin_write().unwrap(), //TODO don't unwrap //TODO begin_read()
            memory: self.memory.clone(),
        };
        R::list(&storage)
    }
}

pub struct PersistentResourcesTransaction<'transaction> {
    db: Mutex<&'transaction mut redb::WriteTransaction>,
    memory: Arc<Mutex<VolatileResourcesStorage>>,
    pub relayed_subscription_events: &'transaction mut RelayedSubscriptionEvents,
}
impl PersistentResourcesTransaction<'_> {
    pub(crate) fn insert<R>(&mut self, id: R::Id, resource: R) -> PersistenceResult<()>
    where R: Resource + Persistable {
        let mut storage = Storage {
            db: &mut self.db.lock().unwrap(),
            memory: self.memory.clone(),
        };
        resource.insert(id, &mut storage)
    }

    pub(crate) fn remove<R>(&mut self, id: R::Id) -> PersistenceResult<Option<R>>
    where R: Resource + Persistable {
        let mut storage = Storage {
            db: &mut self.db.lock().unwrap(),
            memory: self.memory.clone(),
        };
        R::remove(id, &mut storage)
    }

    pub(crate) fn get<R>(&self, id: R::Id) -> PersistenceResult<Option<R>>
    where
        R: Resource + Persistable + Clone
    {
        let storage = Storage {
            db: &mut self.db.lock().unwrap(),
            memory: self.memory.clone(),
        };
        R::get(id, &storage)
    }

    pub(crate) fn list<R>(&self) -> PersistenceResult<HashMap<R::Id, R>>
    where
        R: Resource + Persistable + Clone
    {
        let storage = Storage {
            db: &mut self.db.lock().unwrap(),
            memory: self.memory.clone(),
        };
        R::list(&storage)
    }
}

#[derive(Debug, thiserror::Error)]
enum TransactionPassthroughError {
    #[error("Error returned by Diesel while performing transaction.")]
    Diesel(#[from] diesel::result::Error),
}
