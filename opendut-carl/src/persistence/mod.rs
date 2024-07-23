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

    #[derive(Debug, thiserror::Error)]
    pub enum PersistenceError {
        Custom {
            resource_name: &'static str,
            operation: PersistenceOperation,
            context_messages: Vec<String>,
            id: Option<Id>,
            #[source] source: Cause,
        },
        DieselInternal {
            #[from] source: diesel::result::Error,
        }
    }
    impl PersistenceError {
        pub fn insert<R>(id: Id, cause: impl Into<Cause>) -> Self {
            Self::Custom {
                resource_name: std::any::type_name::<R>(),
                operation: PersistenceOperation::Inserting,
                context_messages: Vec::new(),
                id: Some(id),
                source: cause.into(),
            }
        }
        pub fn get<R>(id: Id, cause: impl Into<Cause>) -> Self {
            Self::Custom {
                resource_name: std::any::type_name::<R>(),
                operation: PersistenceOperation::Getting,
                context_messages: Vec::new(),
                id: Some(id),
                source: cause.into(),
            }
        }
        pub fn list<R>(cause: impl Into<Cause>) -> Self {
            Self::Custom {
                resource_name: std::any::type_name::<R>(),
                operation: PersistenceOperation::Listing,
                context_messages: Vec::new(),
                id: None,
                source: cause.into(),
            }
        }

        pub fn context(mut self, message: impl Into<String>) -> Self {
            match &mut self {
                PersistenceError::Custom { context_messages, .. } => context_messages.push(message.into()),
                PersistenceError::DieselInternal { .. } => unimplemented!(),
            }
            self
        }
    }
    impl Display for PersistenceError {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::Custom { resource_name, operation, context_messages, id, source: _ } => {
                    let id = match &id {
                        Some(id) => format!(" <{id}>"),
                        None => String::from(""),
                    };
                    writeln!(f, "Error while {operation} resource '{resource_name}'{id}")?;

                    for message in context_messages {
                        writeln!(f, "  Context: {message}")?;
                    }
                }
                PersistenceError::DieselInternal { source } => writeln!(f, "{source}")?,
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
