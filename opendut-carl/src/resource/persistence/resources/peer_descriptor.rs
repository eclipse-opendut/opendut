use super::Persistable;
use crate::resource::api::id::ResourceId;
use crate::resource::persistence;
use crate::resource::persistence::error::PersistenceResult;
use crate::resource::persistence::{Db, Key, Memory, TableDefinition, Value};
use opendut_types::peer::{PeerDescriptor, PeerId};
use prost::Message;
use std::collections::HashMap;

const PEER_DESCRIPTOR_TABLE: redb::TableDefinition<Key, Value> = TableDefinition::new("peer_descriptor");


impl Persistable for PeerDescriptor {
    fn insert(self, peer_id: PeerId, _: &mut Memory, db: &Db) -> PersistenceResult<()> {
        let key = persistence::Key::from(ResourceId::<Self>::into_id(peer_id));

        let value = opendut_types::proto::peer::PeerDescriptor::from(self)
            .encode_to_vec();

        let mut table = db.read_write_table(PEER_DESCRIPTOR_TABLE)?;
        table.insert(&key, value)?;

        Ok(())
    }

    fn remove(peer_id: PeerId, _: &mut Memory, db: &Db) -> PersistenceResult<Option<Self>> {
        let key = persistence::Key::from(ResourceId::<Self>::into_id(peer_id));

        let mut table = db.read_write_table(PEER_DESCRIPTOR_TABLE)?;

        let value = table.remove(&key)?
            .map(|value| try_from_bytes(value.value()))
            .transpose()?;

        Ok(value)
    }

    fn get(peer_id: PeerId, _: &Memory, db: &Db) -> PersistenceResult<Option<Self>> {
        let key = persistence::Key::from(ResourceId::<Self>::into_id(peer_id));

        if let Some(table) = db.read_table(PEER_DESCRIPTOR_TABLE)? {
            table.get(&key)?
                .map(|value| try_from_bytes(value.value()))
                .transpose()
        } else {
            Ok(None)
        }
    }

    fn list(_: &Memory, db: &Db) -> PersistenceResult<HashMap<Self::Id, Self>> {
        if let Some(table) = db.read_table(PEER_DESCRIPTOR_TABLE)? {
            table.iter()?
                .map(|value| {
                    let (key, value) = value?;
                    let id = ResourceId::<Self>::from_id(key.value().id);
                    let value = try_from_bytes(value.value())?;

                    Ok((id, value))
                })
                .collect::<PersistenceResult<HashMap<_, _>>>()
        } else {
            Ok(HashMap::default())
        }
    }
}

fn try_from_bytes(bytes: Vec<u8>) -> PersistenceResult<PeerDescriptor> {
    let value = opendut_types::proto::peer::PeerDescriptor::decode(bytes.as_slice())?;
    let value = PeerDescriptor::try_from(value)?;
    Ok(value)
}
