use std::fmt::{Debug, Display, Formatter};
use uuid::Uuid;

#[derive(Debug, thiserror::Error)]
pub struct PersistenceError {
    source: PersistenceErrorKind,
    context_messages: Vec<String>,
}
#[derive(Debug, thiserror::Error)]
pub enum PersistenceErrorKind {
    Custom {
        resource_name: &'static str,
        operation: PersistenceOperation,
        identifier: Option<String>,
        #[source] source: Option<Cause>,
    },
    DieselInternal(#[source] diesel::result::Error),
    JsonSerialization(#[source] serde_json::Error),
    KeyValueStore(#[source] redb::Error),
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
        Self {
            context_messages: Vec::new(),
            source: PersistenceErrorKind::Custom {
                resource_name: std::any::type_name::<R>(),
                operation,
                identifier,
                source: cause.map(Into::into),
            }
        }
    }

    pub fn context(mut self, message: impl Into<String>) -> Self {
        self.context_messages.push(message.into());
        self
    }
}
impl Display for PersistenceError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let Self { source, context_messages } = self;

        writeln!(f, "Error while accessing persistence.")?;
        writeln!(f, "  Cause: {source}")?;

        for message in context_messages {
            writeln!(f, "  Context: {message}")?;
        }

        Ok(())
    }
}
impl Display for PersistenceErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Custom { resource_name, operation, identifier, source } => {
                let identifier = match &identifier {
                    Some(identifier) => format!(" <{identifier}>"),
                    None => String::new(),
                };
                let operation = operation.verb();
                writeln!(f, "Error while {operation} resource '{resource_name}'{identifier}")?;

                source.as_ref().map(|source|
                    writeln!(f, "  Source: {source}")
                ).transpose()?;
            }
            Self::DieselInternal(source) => writeln!(f, "Error internal to Diesel, likely from transaction: {source}")?,
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

pub type PersistenceResult<T> = Result<T, PersistenceError>;
pub trait MapErrToInner<T, E> {
    /// Collapse a PersistenceResult into the value inside of it.
    /// The type of the inner value has to implement `From<PersistenceError>`,
    /// which means it will typically be a `Result` type itself.
    fn map_err_to_inner(self, function: impl FnOnce(PersistenceError) -> E) -> Result<T, E>;
}
impl<T, E> MapErrToInner<T, E> for PersistenceResult<Result<T, E>> {
    fn map_err_to_inner(self, function: impl FnOnce(PersistenceError) -> E) -> Result<T, E> {
        self.unwrap_or_else(|cause|
            Err(function(cause))
        )
    }
}

impl From<serde_json::Error> for PersistenceError {
    fn from(error: serde_json::Error) -> Self {
        PersistenceError {
            source: PersistenceErrorKind::JsonSerialization(error),
            context_messages: vec![],
        }
    }
}
impl From<diesel::result::Error> for PersistenceError {
    fn from(error: diesel::result::Error) -> Self {
        PersistenceError {
            source: PersistenceErrorKind::DieselInternal(error),
            context_messages: vec![],
        }
    }
}
impl From<redb::Error> for PersistenceError {
    fn from(error: redb::Error) -> Self {
        PersistenceError {
            source: PersistenceErrorKind::KeyValueStore(error),
            context_messages: vec![],
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
