use std::sync::{Mutex, MutexGuard};

use crate::resource::storage::volatile::VolatileResourcesStorage;
use diesel::PgConnection;

pub mod database;
pub(crate) mod resources;
mod query;

pub struct Storage<'a> {
    pub db: Db<'a>,
    pub memory: &'a mut Memory,
}
pub struct Db<'a> {
    pub inner: Mutex<&'a mut PgConnection>, //Mutex rather than RwLock, because we share this between threads (i.e. we need it to implement `Sync`)
}
impl<'a> Db<'a> {
    pub fn from_connection(connection: &'a mut PgConnection) -> Db<'a> {
        Self { inner: Mutex::new(connection) }
    }
    pub fn connection(&self) -> MutexGuard<&'a mut PgConnection> {
        self.inner.lock().expect("error while locking mutex for database connection")
    }
}
pub type Memory = VolatileResourcesStorage;

pub(crate) mod error {
    use std::fmt::{Debug, Display, Formatter};
    use uuid::Uuid;

    #[derive(Debug, thiserror::Error)]
    pub enum PersistenceError {
        Custom {
            resource_name: &'static str,
            operation: PersistenceOperation,
            context_messages: Vec<String>,
            identifier: Option<String>,
            #[source] source: Option<Cause>,
        },
        DieselInternal {
            #[from] source: diesel::result::Error,
        },
    }
    impl PersistenceError {
        pub fn insert<R>(identifier: impl Debug, cause: impl Into<Cause>) -> Self {
            Self::new::<R>(Some(identifier), PersistenceOperation::Insert, Some(cause))
        }
        pub fn remove<R>(identifier: impl Debug, cause: impl Into<Cause>) -> Self {
            Self::new::<R>(Some(identifier), PersistenceOperation::Remove, Some(cause))
        }
        pub fn get<R>(identifier: impl Debug, cause: impl Into<Cause>) -> Self {
            Self::new::<R>(Some(identifier), PersistenceOperation::Get, Some(cause))
        }
        pub fn list<R>(cause: impl Into<Cause>) -> Self {
            Self::new::<R>(Option::<Uuid>::None, PersistenceOperation::List, Some(cause))
        }
        pub fn new<R>(identifier: Option<impl Debug>, operation: PersistenceOperation, cause: Option<impl Into<Cause>>) -> Self {
            let identifier = identifier.map(|identifier| format!("{identifier:?}"));
            Self::Custom {
                resource_name: std::any::type_name::<R>(),
                operation,
                context_messages: Vec::new(),
                identifier,
                source: cause.map(Into::into),
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
                Self::Custom { resource_name, operation, context_messages, identifier, source } => {
                    let identifier = match &identifier {
                        Some(identifier) => format!(" <{identifier}>"),
                        None => String::new(),
                    };
                    let operation = operation.verb();
                    writeln!(f, "Error while {operation} resource '{resource_name}'{identifier}")?;

                    for message in context_messages {
                        writeln!(f, "  Context: {message}")?;
                    }
                    source.as_ref().map(|source|
                        writeln!(f, "  Source: {source}")
                    ).transpose()?;
                }
                PersistenceError::DieselInternal { source } => writeln!(f, "Error internal to Diesel, likely from transaction: {source}")?,
            }
            Ok(())
        }
    }

    type Cause = Box<dyn std::error::Error + Send + Sync>;

    #[derive(Clone, Copy, Debug, PartialEq)]
    pub enum PersistenceOperation {
        Insert,
        Remove,
        Get,
        List,
    }
    impl PersistenceOperation {
        fn verb(&self) -> &'static str {
            match self {
                PersistenceOperation::Insert => "inserting",
                PersistenceOperation::Remove => "removing",
                PersistenceOperation::Get => "getting",
                PersistenceOperation::List => "listing",
            }
        }
    }

    pub type PersistenceResult<T> = Result<T, PersistenceError>;
}
