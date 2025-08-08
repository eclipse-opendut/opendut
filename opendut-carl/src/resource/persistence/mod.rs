use std::cmp::Ordering;

use crate::resource::persistence::error::PersistenceResult;
use opendut_types::resources::Id;
use redb::{AccessGuard, ReadableTable, TableError, TypeName};
use uuid::Uuid;

pub(crate) mod error;
pub(crate) mod persistable;

pub type Memory<'transaction> = Db<'transaction>;

pub enum Db<'transaction> {
    Read(&'transaction redb::ReadTransaction),
    ReadWrite(&'transaction mut redb::WriteTransaction),
}
impl Db<'_> {
    pub(super) fn read_table(&self, table: TableDefinition) -> PersistenceResult<Option<ReadTable<'_>>> {
        let open_result = match self {
            Db::Read(transaction) => transaction.open_table(table).map(ReadTable::Read),
            Db::ReadWrite(transaction) => transaction.open_table(table).map(ReadTable::ReadWrite),
        };

        match open_result { //The ReadTransaction does not automatically create the table and rather returns a TableDoesNotExist error
            Ok(table) => Ok(Some(table)),
            Err(cause) => match cause {
                TableError::TableDoesNotExist(_) => Ok(None),
                _ => Err(redb::Error::from(cause))?,
            }
        }
    }

    pub(crate) fn read_write_table(&self, table: TableDefinition) -> PersistenceResult<ReadWriteTable<'_>> {
        match self {
            Db::Read(_) => unimplemented!("Called `.read_write_table()` on a Db::Read() variant."),
            Db::ReadWrite(transaction) => Ok(transaction.open_table(table)?),
        }
    }
}

pub(super) enum ReadTable<'transaction> {
    Read(redb::ReadOnlyTable<Key, Value>),
    ReadWrite(redb::Table<'transaction, Key, Value>),
}
impl ReadTable<'_> {
    pub(crate) fn get(&self, key: &Key) -> redb::Result<Option<AccessGuard<'_, Value>>> {
        match self {
            ReadTable::Read(table) => table.get(key),
            ReadTable::ReadWrite(table) => table.get(key),
        }
    }
    pub(crate) fn iter(&self) -> redb::Result<redb::Range<'_, Key, Value>> {
        match self {
            ReadTable::Read(table) => table.iter(),
            ReadTable::ReadWrite(table) => table.iter(),
        }
    }
}
pub(super) type ReadWriteTable<'a> = redb::Table<'a, Key, Value>;

#[derive(Debug)]
pub struct Key { pub id: Id }

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

pub(super) type Value = Vec<u8>;
pub(super) type TableDefinition<'a> = redb::TableDefinition<'a, Key, Value>;
