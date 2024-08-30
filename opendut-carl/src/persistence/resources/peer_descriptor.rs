use opendut_types::peer::{PeerDescriptor, PeerId};

use super::Persistable;
use crate::persistence::error::PersistenceResult;
use crate::persistence::query::Filter;
use crate::persistence::{query, Storage};

impl Persistable for PeerDescriptor {
    fn insert(self, _peer_id: PeerId, storage: &mut Storage) -> PersistenceResult<()> {
        let mut connection = storage.db.connection();

        //Delete before inserting to ensure that when an update removes
        //list elements we don't leave those elements behind in the database.
        //TODO more efficient solution
        query::peer_descriptor::remove(self.id, &mut connection)?;

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
