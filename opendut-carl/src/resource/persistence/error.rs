use std::fmt::{Debug, Display, Formatter};
use uuid::Uuid;

pub type PersistenceResult<T> = Result<T, PersistenceError>;

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
    JsonSerialization(#[from] serde_json::Error),
    KeyValueStore(#[from] redb::Error),
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
            Self::Custom { context_messages, .. } => context_messages.push(message.into()),
            _ => unimplemented!(),
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
            Self::DieselInternal { source } => writeln!(f, "Error internal to Diesel, likely from transaction: {source}")?,
            Self::JsonSerialization(source) => writeln!(f, "Error while serializing to JSON while storing in or loading from persistence: {source}")?,
            Self::KeyValueStore(source) => writeln!(f, "Error occurred in the key-value store: {source}")?,
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

macro_rules! redb_error_conversion {
    ($type_name:ty) => {
        impl From<$type_name> for PersistenceError {
            fn from(value: $type_name) -> Self {
                let error = redb::Error::from(value);
                PersistenceError::from(error)
            }
        }
    }
}

redb_error_conversion!(redb::CommitError);
redb_error_conversion!(redb::StorageError);
redb_error_conversion!(redb::TableError);
redb_error_conversion!(redb::TransactionError);
