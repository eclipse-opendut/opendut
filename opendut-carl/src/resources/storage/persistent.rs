use crate::persistence::database::ConnectError;
use crate::persistence::error::{PersistenceError, PersistenceResult};
use crate::persistence::resources::Persistable;
use crate::persistence::{Db, Storage};
use crate::resources::storage::volatile::VolatileResourcesStorage;
use crate::resources::storage::{DatabaseConnectInfo, Resource, ResourcesStorageApi};
use diesel::{Connection, PgConnection};
use std::any::Any;
use std::sync::Mutex;
use crate::resources::transaction::RelayedSubscriptionEvents;

pub struct PersistentResourcesStorage {
    db_connection: Mutex<PgConnection>,
    memory: Mutex<VolatileResourcesStorage>,
}
impl PersistentResourcesStorage {
    pub async fn connect(database_connect_info: &DatabaseConnectInfo) -> Result<Self, ConnectError> {
        let db_connection = crate::persistence::database::connect(database_connect_info).await?;
        let db_connection = Mutex::new(db_connection);
        let memory = VolatileResourcesStorage::default();
        let memory = Mutex::new(memory);
        Ok(Self { db_connection, memory })
    }

    pub fn transaction<T, E, F>(&mut self, code: F) -> PersistenceResult<(Result<T, E>, RelayedSubscriptionEvents)>
    where
        F: FnOnce(PersistentResourcesTransaction) -> Result<T, E>,
        E: std::error::Error + Send + Sync + 'static,
    {
        let transaction_result = self.db_connection.lock().unwrap().transaction::<_, TransactionPassthroughError, _>(|connection| {
            let mut memory = self.memory.lock().unwrap();
            let mut relayed_subscription_events = RelayedSubscriptionEvents::default();

            let transaction = PersistentResourcesTransaction {
                db_connection: Mutex::new(connection),
                memory: Mutex::new(&mut memory),
                relayed_subscription_events: &mut relayed_subscription_events,
            };

            let result = code(transaction);
            match result {
                Ok(ok) => Ok((ok, relayed_subscription_events)),
                Err(error) => Err(TransactionPassthroughError::Passthrough(Box::new(error))), //passthrough via an Err-value to trigger transaction rollback
            }
        });

        match transaction_result {
            Ok((result, relayed_subscription_events)) => Ok((Ok(result), relayed_subscription_events)),
            Err(TransactionPassthroughError::Passthrough(error)) => {
                let error = error.downcast::<E>()
                    .expect("should be error of type E, like we handed it out from the transaction");
                Ok((Err(*error), RelayedSubscriptionEvents::default())) //FIXME can we omit RelayedSubscriptionEvents at compile-time?
            }
            Err(TransactionPassthroughError::Diesel(source)) => Err(PersistenceError::DieselInternal { source }),
        }
    }
}
impl ResourcesStorageApi for PersistentResourcesStorage {
    fn insert<R>(&mut self, id: R::Id, resource: R) -> PersistenceResult<()>
    where R: Resource + Persistable {
        let mut db = self.db_connection.lock().unwrap();
        let db = Db::from_connection(&mut db);
        let mut storage = Storage { db, memory: &mut self.memory.lock().unwrap() };
        resource.insert(id, &mut storage)
    }

    fn remove<R>(&mut self, id: R::Id) -> PersistenceResult<Option<R>>
    where R: Resource + Persistable {
        let mut db = self.db_connection.lock().unwrap();
        let db = Db::from_connection(&mut db);
        let mut storage = Storage { db, memory: &mut self.memory.lock().unwrap() };
        R::remove(id, &mut storage)
    }

    fn get<R>(&self, id: R::Id) -> PersistenceResult<Option<R>>
    where R: Resource + Persistable + Clone {
        let mut db = self.db_connection.lock().unwrap();
        let db = Db::from_connection(&mut db);
        let storage = Storage { db, memory: &mut self.memory.lock().unwrap() };
        R::get(id, &storage)
    }

    fn list<R>(&self) -> PersistenceResult<Vec<R>>
    where R: Resource + Persistable + Clone {
        let mut db = self.db_connection.lock().unwrap();
        let db = Db::from_connection(&mut db);
        let storage = Storage { db, memory: &mut self.memory.lock().unwrap() };
        R::list(&storage)
    }
}


pub struct PersistentResourcesTransaction<'transaction> {
    db_connection: Mutex<&'transaction mut PgConnection>,
    memory: Mutex<&'transaction mut VolatileResourcesStorage>,
    pub relayed_subscription_events: &'transaction mut RelayedSubscriptionEvents,
}
impl ResourcesStorageApi for PersistentResourcesTransaction<'_> {
    fn insert<R>(&mut self, id: R::Id, resource: R) -> PersistenceResult<()>
    where R: Resource + Persistable {
        let mut db = self.db_connection.lock().unwrap();
        let db = Db::from_connection(&mut db);
        let mut storage = Storage { db, memory: &mut self.memory.lock().unwrap() };
        resource.insert(id, &mut storage)
    }

    fn remove<R>(&mut self, id: R::Id) -> PersistenceResult<Option<R>>
    where R: Resource + Persistable {
        let mut db = self.db_connection.lock().unwrap();
        let db = Db::from_connection(&mut db);
        let mut storage = Storage { db, memory: &mut self.memory.lock().unwrap() };
        R::remove(id, &mut storage)
    }

    fn get<R>(&self, id: R::Id) -> PersistenceResult<Option<R>>
    where
        R: Resource + Persistable + Clone
    {
        let mut db = self.db_connection.lock().unwrap();
        let db = Db::from_connection(&mut db);
        let storage = Storage { db, memory: &mut self.memory.lock().unwrap() };
        R::get(id, &storage)
    }

    fn list<R>(&self) -> PersistenceResult<Vec<R>>
    where
        R: Resource + Persistable + Clone
    {
        let mut db = self.db_connection.lock().unwrap();
        let db = Db::from_connection(&mut db);
        let storage = Storage { db, memory: &mut self.memory.lock().unwrap() };
        R::list(&storage)
    }
}

#[derive(Debug, thiserror::Error)]
enum TransactionPassthroughError {
    #[error("Error returned by Diesel while performing transaction.")]
    Diesel(#[from] diesel::result::Error),
    #[error("Error returned by the code executed within the transaction.")]
    Passthrough(Box<dyn Any + Send + Sync>),
}
