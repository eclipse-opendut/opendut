use super::Persistable;
use crate::resource::api::id::ResourceId;
use crate::resource::persistence;
use crate::resource::persistence::error::PersistenceResult;
use crate::resource::persistence::{Db, Memory, TableDefinition};
use opendut_types::peer::executor::ExecutorDescriptors;
use opendut_types::peer::{PeerDescriptor, PeerId, PeerLocation, PeerName, PeerNetworkDescriptor};
use opendut_types::topology::Topology;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

impl Persistable for PeerDescriptor {

    fn insert(self, peer_id: PeerId, _: &mut Memory, db: &Db) -> PersistenceResult<()> {
        let key = persistence::Key::from(ResourceId::<Self>::into_id(peer_id));

        let mut table = db.read_write_table(PEER_DESCRIPTOR_TABLE)?;

        let value = {
            let PeerDescriptor { id, name, location, network, topology, executors } = self;
            SerializablePeerDescriptor {
                id, name, location, network, topology, executors,
            }
        };
        let value = serde_json::to_string(&value)?;

        table.insert(&key, value)?;

        Ok(())
    }

    fn remove(peer_id: PeerId, _: &mut Memory, db: &Db) -> PersistenceResult<Option<Self>> {
        let key = persistence::Key::from(ResourceId::<Self>::into_id(peer_id));

        let mut table = db.read_write_table(PEER_DESCRIPTOR_TABLE)?;

        let value = table.remove(&key)?
            .map(|value| {
                let peer_descriptor = serde_json::from_str::<SerializablePeerDescriptor>(&value.value())?;

                let SerializablePeerDescriptor { id, name, location, network, topology, executors } = peer_descriptor;
                PersistenceResult::Ok(PeerDescriptor { id, name, location, network, topology, executors })
            })
            .transpose()?;

        Ok(value)
    }

    fn get(peer_id: PeerId, _: &Memory, db: &Db) -> PersistenceResult<Option<Self>> {
        let key = persistence::Key::from(ResourceId::<Self>::into_id(peer_id));

        if let Some(table) = db.read_table(PEER_DESCRIPTOR_TABLE)? {
            table.get(&key)?
                .map(|value| {
                    let peer_descriptor = serde_json::from_str::<SerializablePeerDescriptor>(&value.value())?;

                    let SerializablePeerDescriptor { id, name, location, network, topology, executors } = peer_descriptor;
                    PersistenceResult::Ok(PeerDescriptor { id, name, location, network, topology, executors })
                })
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

                    let peer_descriptor = serde_json::from_str::<SerializablePeerDescriptor>(&value.value())?;
                    let peer_descriptor = {
                        let SerializablePeerDescriptor { id, name, location, network, topology, executors } = peer_descriptor;
                        PeerDescriptor { id, name, location, network, topology, executors }
                    };

                    Ok((id, peer_descriptor))
                })
                .collect::<PersistenceResult<HashMap<_, _>>>()
        } else {
            Ok(HashMap::default())
        }
    }
}

const PEER_DESCRIPTOR_TABLE: TableDefinition = TableDefinition::new("peer_descriptor");

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct SerializablePeerDescriptor { //TODO From-implementation //TODO version-field, if nothing exists natively in redb
    pub id: PeerId,
    pub name: PeerName,
    pub location: Option<PeerLocation>,
    pub network: PeerNetworkDescriptor,
    pub topology: Topology,
    pub executors: ExecutorDescriptors,
}
