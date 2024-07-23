use std::sync::Mutex;

use diesel::PgConnection;
use crate::resources::storage::volatile::VolatileResourcesStorage;

pub mod database;
pub mod model;

pub struct Storage {
    pub db: Db,
    pub memory: Memory,
}
pub type Db = Mutex<PgConnection>; //Mutex rather than RwLock, because we share this between threads (i.e. we need it to implement `Sync`)
pub type Memory = VolatileResourcesStorage;

pub mod error {
    use std::fmt::{Display, Formatter};
    use crate::resources::resource::Resource;

    #[derive(Debug, thiserror::Error)]
    pub struct PersistenceError {
        resource_name: &'static str,
        operation: PersistenceOperation,
        context_message: Option<String>,
        #[source] source: Cause,
    }
    impl PersistenceError {
        pub fn insert<R: Resource>(cause: impl Into<Cause>) -> Self {
            Self {
                resource_name: std::any::type_name::<R>(),
                operation: PersistenceOperation::Inserting,
                context_message: None,
                source: cause.into(),
            }
        }
        pub fn get<R: Resource>(cause: impl Into<Cause>) -> Self {
            Self {
                resource_name: std::any::type_name::<R>(),
                operation: PersistenceOperation::Getting,
                context_message: None,
                source: cause.into(),
            }
        }
        pub fn list<R: Resource>(cause: impl Into<Cause>) -> Self {
            Self {
                resource_name: std::any::type_name::<R>(),
                operation: PersistenceOperation::Listing,
                context_message: None,
                source: cause.into(),
            }
        }

        pub fn context(mut self, message: impl Into<String>) -> Self {
            self.context_message = Some(message.into());
            self
        }
    }
    impl Display for PersistenceError {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            let message = match &self.context_message {
                Some(message) => format!(": {message}"),
                None => String::from(""),
            };
            write!(f, "Error while {operation} resource '{resource}'{message}", operation=self.operation, resource=self.resource_name)
        }
    }

    type Cause = Box<dyn std::error::Error + Send + Sync>;

    #[derive(Debug)]
    pub enum PersistenceOperation {
        Inserting,
        Getting,
        Listing,
    }
    impl Display for PersistenceOperation {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            let string = match self {
                PersistenceOperation::Inserting => "inserting",
                PersistenceOperation::Getting => "getting",
                PersistenceOperation::Listing => "listing",
            };
            write!(f, "{string}")
        }
    }

    pub type PersistenceResult<T> = Result<T, PersistenceError>;
}
