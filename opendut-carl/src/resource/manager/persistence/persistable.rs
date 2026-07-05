use std::fmt::Debug;

use opendut_util::proto::ConversionError;
use prost::Message;

use crate::resource::types::Resource;

use super::TableDefinition;
use super::error::PersistenceResult;

pub trait Persistable: Send + Sync + Sized + Debug + Resource {
	type Proto: Message + Default + From<Self> + TryInto<Self, Error=ConversionError>;
	const TABLE: &'static str;
	const STORAGE: StorageKind;


	const TABLE_DEFINITION: TableDefinition<'_> = TableDefinition::new(Self::TABLE);

	fn try_from_bytes(bytes: Vec<u8>) -> PersistenceResult<Self> {
		let value = Self::Proto::decode(bytes.as_slice())?;
		let value: Self = value.try_into()?;
		Ok(value)
	}
}

pub enum StorageKind { Persistent, Volatile }
