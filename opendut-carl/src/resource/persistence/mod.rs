use std::cmp::Ordering;
use std::sync::{Arc, Mutex};

use crate::resource::storage::volatile::VolatileResourcesStorage;
use redb::{AccessGuard, ReadableTable, TableError, TypeName};
use uuid::Uuid;
use opendut_types::resources::Id;
use crate::resource::persistence::error::PersistenceResult;

pub mod database;
pub(crate) mod resources;
mod query;

pub type Memory = Arc<Mutex<VolatileResourcesStorage>>;

pub enum Db<'transaction> {
    Read(&'transaction redb::ReadTransaction),
    ReadWrite(&'transaction mut redb::WriteTransaction),
}
impl Db<'_> {
    fn read_table(&self, table: TableDefinition) -> PersistenceResult<Option<ReadTable>> {
        let open_result = match self {
            Db::Read(transaction) => transaction.open_table(table).map(ReadTable::Read),
            Db::ReadWrite(transaction) => transaction.open_table(table).map(ReadTable::ReadWrite),
        };

        match open_result { //The ReadTransaction does not automatically create the table and rather returns a TableDoesNotExist error
            Ok(table) => Ok(Some(table)),
            Err(cause) => match cause {
                TableError::TableDoesNotExist(_) => Ok(None),
                _ => Err(cause)?,
            }
        }
    }

    fn read_write_table(&self, table: TableDefinition) -> PersistenceResult<ReadWriteTable> {
        match self {
            Db::Read(_) => unimplemented!("Called `.read_write_table()` on a Db::Read() variant."),
            Db::ReadWrite(transaction) => transaction.open_table(table).map_err(Into::into),
        }
    }
}

pub(super) enum ReadTable<'transaction> {
    Read(redb::ReadOnlyTable<Key, Value>),
    ReadWrite(redb::Table<'transaction, Key, Value>),
}
impl ReadTable<'_> {
    fn get(&self, key: &Key) -> redb::Result<Option<AccessGuard<Value>>> {
        match self {
            ReadTable::Read(table) => table.get(key),
            ReadTable::ReadWrite(table) => table.get(key),
        }
    }
    fn iter(&self) -> redb::Result<redb::Range<Key, Value>> {
        match self {
            ReadTable::Read(table) => table.iter(),
            ReadTable::ReadWrite(table) => table.iter(),
        }
    }
}
pub(super) type ReadWriteTable<'a> = redb::Table<'a, Key, Value>;

#[derive(Debug)]
pub(super) struct Key { pub id: Id }

impl redb::Key for Key {
    fn compare(data1: &[u8], data2: &[u8]) -> Ordering {
        let data1 = Uuid::from_slice(data1)
            .expect("A UUID which was previously saved to bytes should be loadable from bytes.");
        let data2 = Uuid::from_slice(data2)
            .expect("A UUID which was previously saved to bytes should be loadable from bytes.");

        data1.cmp(&data2)
    }
}
impl redb::Value for Key {
    type SelfType<'a> = Self
    where
        Self: 'a;

    type AsBytes<'a> = [u8; 16]
    where
        Self: 'a;

    fn fixed_width() -> Option<usize> {
        Some(16)
    }

    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
    where
        Self: 'a
    {
        let uuid = Uuid::from_slice(data)
            .expect("A PersistenceId which was previously saved to bytes should be loadable from bytes.");
        Key { id: Id::from(uuid) }
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
    where
        Self: 'b
    {
        let Key { id } = value;
        *id.value().as_bytes()
    }

    fn type_name() -> TypeName {
        TypeName::new("opendut_persistence_id")
    }
}
impl From<Id> for Key {
    fn from(value: Id) -> Self {
        Key { id: value }
    }
}

pub(super) type Value = String;
pub(super) type TableDefinition<'a> = redb::TableDefinition<'a, Key, Value>;


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
        Table(#[from] redb::TableError),
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
                Self::DieselInternal { .. } => unimplemented!(),
                Self::Table(_) => unimplemented!(),
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
                Self::Table(source) => writeln!(f, "Error while interacting with key-value table: {source}")?,
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
