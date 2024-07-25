use std::sync::Mutex;

use diesel::PgConnection;
use crate::resources::storage::volatile::VolatileResourcesStorage;

pub mod database;
pub(crate) mod model;

pub struct Storage {
    pub db: Db,
    pub memory: Memory,
}
pub(crate) type Db = Mutex<PgConnection>; //Mutex rather than RwLock, because we share this between threads (i.e. we need it to implement `Sync`)
pub(crate) type Memory = VolatileResourcesStorage;

pub(crate) mod error {
    use std::fmt::{Display, Formatter};
    use opendut_types::resources::Id;
    use crate::resources::resource::Resource;

    #[derive(Debug, thiserror::Error)]
    pub struct PersistenceError {
        resource_name: &'static str,
        operation: PersistenceOperation,
        context_messages: Vec<String>,
        id: Option<Id>,
        #[source] source: Cause,
    }
    impl PersistenceError {
        pub fn insert<R: Resource>(id: Id, cause: impl Into<Cause>) -> Self {
            Self {
                resource_name: std::any::type_name::<R>(),
                operation: PersistenceOperation::Inserting,
                context_messages: Vec::new(),
                id: Some(id),
                source: cause.into(),
            }
        }
        pub fn get<R: Resource>(id: Id, cause: impl Into<Cause>) -> Self {
            Self {
                resource_name: std::any::type_name::<R>(),
                operation: PersistenceOperation::Getting,
                context_messages: Vec::new(),
                id: Some(id),
                source: cause.into(),
            }
        }
        pub fn list<R: Resource>(cause: impl Into<Cause>) -> Self {
            Self {
                resource_name: std::any::type_name::<R>(),
                operation: PersistenceOperation::Listing,
                context_messages: Vec::new(),
                id: None,
                source: cause.into(),
            }
        }

        pub fn context(mut self, message: impl Into<String>) -> Self {
            self.context_messages.push(message.into());
            self
        }
    }
    impl Display for PersistenceError {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            let operation = self.operation;
            let resource = self.resource_name;
            let id = match &self.id {
                Some(id) => format!(" <{id}>"),
                None => String::from(""),
            };
            writeln!(f, "Error while {operation} resource '{resource}'{id}")?;

            for message in &self.context_messages {
                writeln!(f, "  Context: {message}")?;
            }
            Ok(())
        }
    }

    type Cause = Box<dyn std::error::Error + Send + Sync>;

    #[derive(Clone, Copy, Debug, PartialEq)]
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
