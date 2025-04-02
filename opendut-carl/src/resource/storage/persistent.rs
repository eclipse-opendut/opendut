use crate::resource::api::resources::RelayedSubscriptionEvents;
use crate::resource::persistence::error::PersistenceResult;
use crate::resource::persistence::resources::Persistable;
use crate::resource::persistence::{Db, Memory};
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
    memory: Memory,
}
impl PersistentResourcesStorage {
    pub async fn connect(database_connect_info: &DatabaseConnectInfo) -> Result<Self, ConnectError> {
        // let _ = crate::resource::persistence::database::connect(database_connect_info).await?; //TODO remove or use for migration

        let file = &database_connect_info.file;

        if let Some(parent_dir) = file.parent() {
            fs::create_dir_all(parent_dir)
                .map_err(|source| ConnectError::DatabaseDirCreate { dir: parent_dir.to_owned(), source })?;
        }

        let db = redb::Database::create(file)
            .map_err(|source| ConnectError::DatabaseCreate { file: file.to_owned(), source })?;
        debug!("Database file opened from: {file:?}");

        let memory = Arc::new(Mutex::new(
            VolatileResourcesStorage::default()
        ));

        Ok(Self { db, memory })
    }

    pub async fn resources<T, F>(&self, code: F) -> T
    where
        F: AsyncFnOnce(PersistentResourcesTransaction) -> T,
    {
        let mut relayed_subscription_events = RelayedSubscriptionEvents::default();

        let transaction = self.db.begin_read().unwrap(); //TODO don't unwrap
        let result = {
            let transaction = PersistentResourcesTransaction {
                db: Db::Read(&transaction),
                memory: self.memory.clone(),
                relayed_subscription_events: &mut relayed_subscription_events,
            };

            code(transaction).await
        };

        debug_assert!(relayed_subscription_events.is_empty(), "Read-only storage operations should not trigger any subscription events.");

        result
    }

    pub async fn resources_mut<T, E, F>(&mut self, code: F) -> PersistenceResult<(Result<T, E>, RelayedSubscriptionEvents)>
    where
        F: AsyncFnOnce(PersistentResourcesTransaction) -> Result<T, E>,
        E: Display,
    {
        let mut relayed_subscription_events = RelayedSubscriptionEvents::default();
        let mut transaction = self.db.begin_write()?;
        let result = {
            let persistent_transaction = PersistentResourcesTransaction {
                db: Db::ReadWrite(&mut transaction),
                memory: self.memory.clone(),
                relayed_subscription_events: &mut relayed_subscription_events,
            };

            code(persistent_transaction).await
        };

        match &result {
            Ok(_) => {
                transaction.commit()?;
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
        let mut transaction = self.db.begin_write()?;
        let result = resource.insert(id, &mut self.memory.clone(), &Db::ReadWrite(&mut transaction));
        transaction.commit()?;
        //TODO emit subscription events
        result
    }

    fn remove<R>(&mut self, id: R::Id) -> PersistenceResult<Option<R>>
    where R: Resource + Persistable {
        let mut transaction = self.db.begin_write()?;
        let result = R::remove(id, &mut self.memory.clone(), &Db::ReadWrite(&mut transaction));
        transaction.commit()?;
        //TODO emit subscription events
        result
    }

    fn get<R>(&self, id: R::Id) -> PersistenceResult<Option<R>>
    where R: Resource + Persistable + Clone {
        let transaction = self.db.begin_read()?;
        R::get(id, &self.memory.clone(), &Db::Read(&transaction))
    }

    fn list<R>(&self) -> PersistenceResult<HashMap<R::Id, R>>
    where R: Resource + Persistable + Clone {
        let transaction = self.db.begin_read()?;
        R::list(&self.memory.clone(), &Db::Read(&transaction))
    }
}

pub struct PersistentResourcesTransaction<'transaction> {
    db: Db<'transaction>,
    memory: Memory,
    pub relayed_subscription_events: &'transaction mut RelayedSubscriptionEvents,
}
impl ResourcesStorageApi for PersistentResourcesTransaction<'_> {
    fn insert<R>(&mut self, id: R::Id, resource: R) -> PersistenceResult<()>
    where R: Resource + Persistable {
        resource.insert(id, &mut self.memory.clone(), &self.db)
        //TODO emit subscription events
    }

    fn remove<R>(&mut self, id: R::Id) -> PersistenceResult<Option<R>>
    where R: Resource + Persistable {
        R::remove(id, &mut self.memory.clone(), &self.db)
        //TODO emit subscription events
    }

    fn get<R>(&self, id: R::Id) -> PersistenceResult<Option<R>>
    where
        R: Resource + Persistable + Clone
    {
        R::get(id, &self.memory.clone(), &self.db)
    }

    fn list<R>(&self) -> PersistenceResult<HashMap<R::Id, R>>
    where
        R: Resource + Persistable + Clone
    {
        R::list(&self.memory.clone(), &self.db)
    }
}

#[derive(Debug, thiserror::Error)]
enum TransactionPassthroughError {
    #[error("Error returned by Diesel while performing transaction.")]
    Diesel(#[from] diesel::result::Error),
}
