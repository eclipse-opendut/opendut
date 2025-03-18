use opendut_types::peer::{PeerDescriptor, PeerId};

use super::Persistable;
use crate::resource::persistence::error::PersistenceResult;
use crate::resource::persistence::query::Filter;
use crate::resource::persistence::{query, Storage};

impl Persistable for PeerDescriptor {
    fn insert(self, _peer_id: PeerId, storage: &mut Storage) -> PersistenceResult<()> {
        let mut connection = storage.db.connection();

        query::peer_descriptor::insert(self, &mut connection)
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
