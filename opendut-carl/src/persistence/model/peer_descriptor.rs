use std::ops::DerefMut;
use diesel::RunQueryDsl;
use uuid::Uuid;

use opendut_types::peer::{PeerDescriptor, PeerId, PeerLocation, PeerName};
use opendut_types::peer::executor::ExecutorDescriptors;
use crate::persistence::database::schema;
use crate::persistence::database::Db;
use super::{Persistable, PersistableConversionError};

#[derive(Debug, diesel::Queryable, diesel::Selectable, diesel::Insertable)]
#[diesel(table_name = schema::peer)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PersistablePeerDescriptor {
    pub id: Uuid,
    pub name: String,
    pub location: Option<String>,
}
impl Persistable<PeerDescriptor> for PersistablePeerDescriptor {
    fn insert(&self, db: Db) -> Option<Self> {
        diesel::insert_into(schema::peer::table)
            .values(self)
            // .returning(PersistablePeerDescriptor::as_returning()) //TODO
            .execute(db.lock().unwrap().deref_mut()) //TODO don't unwrap() //TODO use .get_result() instead and return the value
            .expect("Error inserting PeerDescriptor into database"); //TODO don't expect()

        None //TODO
    }
}

impl From<PeerDescriptor> for PersistablePeerDescriptor {
    fn from(value: PeerDescriptor) -> Self {
        Self {
            id: value.id.uuid,
            name: value.name.value(),
            location: value.location.map(|location| location.value())
        }
    }
}
impl TryFrom<PersistablePeerDescriptor> for PeerDescriptor {
    type Error = PersistableConversionError<PersistablePeerDescriptor, PeerDescriptor>;

    fn try_from(value: PersistablePeerDescriptor) -> Result<Self, Self::Error> {

        let name = PeerName::try_from(value.name)
            .map_err(|cause| PersistableConversionError::new(Box::new(cause)))?;

        let location = value.location
            .map(PeerLocation::try_from)
            .transpose()
            .map_err(|cause| PersistableConversionError::new(Box::new(cause)))?;

        Ok(Self {
            id: PeerId::from(value.id),
            name,
            location,
            network: Default::default(), //TODO
            topology: Default::default(), //TODO
            executors: ExecutorDescriptors { executors: Default::default() }, //TODO
        })
    }
}
