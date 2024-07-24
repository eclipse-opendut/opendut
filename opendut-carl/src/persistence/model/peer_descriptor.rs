use std::ops::DerefMut;

use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl, SelectableHelper};
use uuid::Uuid;

use opendut_types::peer::{PeerDescriptor, PeerId, PeerLocation, PeerName};
use opendut_types::peer::executor::ExecutorDescriptors;
use opendut_types::resources::Id;

use crate::persistence::database::schema;
use crate::persistence::error::{PersistenceError, PersistenceResult};
use crate::persistence::Storage;

use super::{Persistable, PersistableConversionError, PersistableModel};

impl Persistable for PeerDescriptor {
    fn insert(self, id: Id, storage: &mut Storage) -> PersistenceResult<()> {
        let persistable = PersistablePeerDescriptor::from(self);

        diesel::insert_into(schema::peer_descriptor::table)
            .values(&persistable)
            .on_conflict(schema::peer_descriptor::peer_id)
            .do_update()
            .set(&persistable)
            .execute(storage.db.lock().unwrap().deref_mut())
            .map_err(|cause| PersistenceError::insert::<Self>(id, cause))?;
        Ok(())
    }

    fn get(id: Id, storage: &Storage) -> PersistenceResult<Option<Self>> {
        let persistable = schema::peer_descriptor::table
            .filter(schema::peer_descriptor::peer_id.eq(id.value()))
            .select(PersistablePeerDescriptor::as_select())
            .first(storage.db.lock().unwrap().deref_mut())
            .optional()
            .map_err(|cause| PersistenceError::get::<Self>(id, cause))?;

        let result = persistable
            .map(TryInto::try_into)
            .transpose()
            .map_err(|cause| PersistenceError::get::<Self>(id, cause).context("Failed to convert from PersistablePeerDescriptor."))?;
        Ok(result)
    }

    fn list(storage: &Storage) -> PersistenceResult<Vec<Self>> {
        let persistables = schema::peer_descriptor::table
            .select(PersistablePeerDescriptor::as_select())
            .get_results(storage.db.lock().unwrap().deref_mut())
            .map_err(PersistenceError::list::<Self>)?;

        let result = persistables
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|cause| PersistenceError::list::<Self>(cause).context("Failed to convert from list of PersistablePeerDescriptors."))?;
        Ok(result)
    }
}

#[derive(Debug, diesel::Queryable, diesel::Selectable, diesel::Insertable, diesel::AsChangeset)]
#[diesel(table_name = schema::peer_descriptor)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct PersistablePeerDescriptor {
    pub peer_id: Uuid,
    pub name: String,
    pub location: Option<String>,
}
impl PersistableModel<PeerDescriptor> for PersistablePeerDescriptor { }

impl From<PeerDescriptor> for PersistablePeerDescriptor {
    fn from(value: PeerDescriptor) -> Self {
        Self {
            peer_id: value.id.uuid,
            name: value.name.value(),
            location: value.location.map(|location| location.value())
        }
    }
}
impl TryInto<PeerDescriptor> for PersistablePeerDescriptor {
    type Error = PersistableConversionError<PersistablePeerDescriptor, PeerDescriptor>;

    fn try_into(self) -> Result<PeerDescriptor, Self::Error> {

        let name = PeerName::try_from(self.name)
            .map_err(|cause| PersistableConversionError::new(Box::new(cause)))?;

        let location = self.location
            .map(PeerLocation::try_from)
            .transpose()
            .map_err(|cause| PersistableConversionError::new(Box::new(cause)))?;

        Ok(PeerDescriptor {
            id: PeerId::from(self.peer_id),
            name,
            location,
            network: Default::default(), //TODO
            topology: Default::default(), //TODO
            executors: ExecutorDescriptors { executors: Default::default() }, //TODO
        })
    }
}
