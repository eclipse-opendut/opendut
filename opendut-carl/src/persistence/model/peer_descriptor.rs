use std::ops::DerefMut;

use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl, SelectableHelper};
use uuid::Uuid;

use opendut_types::peer::{PeerDescriptor, PeerId, PeerLocation, PeerName};
use opendut_types::peer::executor::ExecutorDescriptors;
use opendut_types::resources::Id;

use crate::persistence::database::schema;
use crate::persistence::Storage;
use super::{Persistable, PersistableConversionError, PersistableModel};

impl Persistable for PeerDescriptor {
    fn insert(self, id: Id, storage: &mut Storage) {
        let persistable = PersistablePeerDescriptor::from(self);

        diesel::insert_into(schema::peer::table)
            .values(persistable)
            .execute(storage.db.lock().unwrap().deref_mut()) //TODO don't unwrap()
            .expect("Error inserting PeerDescriptor into database"); //TODO don't expect()
    }

    fn get(id: Id, storage: &Storage) -> Option<Self> {
        let persistable = schema::peer::table
            .filter(schema::peer::id.eq(id.value()))
            .select(PersistablePeerDescriptor::as_select())
            .first(storage.db.lock().unwrap().deref_mut()) //TODO don't unwrap()
            .optional()
            .expect("Error getting PeerDescriptor from database"); //TODO don't expect()

        persistable
            .map(TryInto::try_into)
            .transpose()
            .expect("Failed to convert from PersistablePeerDescriptor.") //TODO don't expect
    }

    fn list(storage: &Storage) -> Vec<Self> {
        let persistables = schema::peer::table
            .select(PersistablePeerDescriptor::as_select())
            .get_results(storage.db.lock().unwrap().deref_mut()) //TODO don't unwrap()
            .expect("Error getting list of PeerDescriptors from database"); //TODO don't expect()

        persistables
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<_>, _>>()
            .expect("Failed to convert from list of PersistablePeerDescriptors.") //TODO don't expect
    }
}

#[derive(Debug, diesel::Queryable, diesel::Selectable, diesel::Insertable)]
#[diesel(table_name = schema::peer)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct PersistablePeerDescriptor {
    pub id: Uuid,
    pub name: String,
    pub location: Option<String>,
}
impl PersistableModel<PeerDescriptor> for PersistablePeerDescriptor { }

impl From<PeerDescriptor> for PersistablePeerDescriptor {
    fn from(value: PeerDescriptor) -> Self {
        Self {
            id: value.id.uuid,
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
            id: PeerId::from(self.id),
            name,
            location,
            network: Default::default(), //TODO
            topology: Default::default(), //TODO
            executors: ExecutorDescriptors { executors: Default::default() }, //TODO
        })
    }
}
