use diesel::Connection;
use opendut_types::topology::{DeviceDescriptor, DeviceId};

use crate::persistence::error::{PersistenceError, PersistenceResult};
use crate::persistence::query::Filter;
use crate::persistence::resources::Persistable;
use crate::persistence::{query, Storage};

impl Persistable for DeviceDescriptor {
    fn insert(self, _device_id: DeviceId, storage: &mut Storage) -> PersistenceResult<()> {
        storage.db.connection().transaction::<_, PersistenceError, _>(|connection| {
            //Delete before inserting to ensure that when an update removes
            //list elements we don't leave those elements behind in the database.
            //TODO more efficient solution
            query::device_descriptor::remove(self.id, connection)?;

            query::device_descriptor::insert(self, connection)
        })
    }

    fn remove(device_id: DeviceId, storage: &mut Storage) -> PersistenceResult<Option<Self>> {
        query::device_descriptor::remove(device_id, &mut storage.db.connection())
    }

    fn get(device_id: DeviceId, storage: &Storage) -> PersistenceResult<Option<Self>> {
        let result = query::device_descriptor::list(Filter::By(device_id), &mut storage.db.connection())?
            .first().cloned();
        Ok(result)
    }

    fn list(storage: &Storage) -> PersistenceResult<Vec<Self>> {
        query::device_descriptor::list(Filter::Not, &mut storage.db.connection())
    }
}
