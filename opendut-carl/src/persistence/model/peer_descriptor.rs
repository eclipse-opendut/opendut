use diesel::Connection;

use opendut_types::peer::{PeerDescriptor, PeerId};

use super::{query, Persistable};
use crate::persistence::error::{PersistenceError, PersistenceResult};
use crate::persistence::model::query::Filter;
use crate::persistence::Storage;

impl Persistable for PeerDescriptor {
    fn insert(self, _peer_id: PeerId, storage: &mut Storage) -> PersistenceResult<()> {
        storage.db.connection().transaction::<_, PersistenceError, _>(|connection| {
            query::peer_descriptor::insert(self, connection)
        })
    }

    fn remove(peer_id: PeerId, storage: &mut Storage) -> PersistenceResult<Option<Self>> {
        query::peer_descriptor::remove(peer_id, &mut storage.db.connection())
    }

    fn get(peer_id: PeerId, storage: &Storage) -> PersistenceResult<Option<Self>> {
        let result = query::peer_descriptor::list(Filter::By(peer_id), &mut storage.db.connection())?
            .first().cloned();
        Ok(result)
    }

    fn list(storage: &Storage) -> PersistenceResult<Vec<Self>> {
        query::peer_descriptor::list(Filter::Not, &mut storage.db.connection())
    }
}
