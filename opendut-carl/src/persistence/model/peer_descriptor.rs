use std::ops::DerefMut;
use diesel::{ExpressionMethods, OptionalExtension, PgConnection, QueryDsl, RunQueryDsl, SelectableHelper};
use uuid::Uuid;

use opendut_types::peer::{PeerDescriptor, PeerId, PeerLocation, PeerName};
use opendut_types::peer::executor::ExecutorDescriptors;
use opendut_types::resources::Id;

use crate::persistence::database::schema;
use crate::persistence::error::{PersistenceError, PersistenceResult};
use crate::persistence::Storage;

use super::{Persistable, PersistableConversionError};

impl Persistable for PeerDescriptor {
    fn insert(self, id: Id, storage: &mut Storage) -> PersistenceResult<()> {
        let PeerDescriptor { id: peer_id, name, location, network, topology, executors } = self;

        PersistablePeerDescriptor {
            peer_id: peer_id.uuid,
            name: name.value(),
            location: location.map(|location| location.value())
        }.insert(id, storage.db.lock().unwrap().deref_mut())?;

        //TODO persist other fields

        Ok(())
    }

    fn get(id: Id, storage: &Storage) -> PersistenceResult<Option<Self>> {
        let persistable = PersistablePeerDescriptor::get(id, storage.db.lock().unwrap().deref_mut())?;

        let result = persistable
            .map(TryInto::try_into)
            .transpose()
            .map_err(|cause| PersistenceError::get::<Self>(id, cause).context("Failed to convert from PersistablePeerDescriptor."))?;
        Ok(result)
    }

    fn list(storage: &Storage) -> PersistenceResult<Vec<Self>> {
        let persistables = PersistablePeerDescriptor::list(storage.db.lock().unwrap().deref_mut())?;

        let result = persistables
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|cause| PersistenceError::list::<Self>(cause).context("Failed to convert from list of PersistablePeerDescriptors."))?;
        Ok(result)
    }
}



#[derive(Clone, Debug, PartialEq, diesel::Queryable, diesel::Selectable, diesel::Insertable, diesel::AsChangeset)]
#[diesel(table_name = schema::peer_descriptor)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct PersistablePeerDescriptor {
    pub peer_id: Uuid,
    pub name: String,
    pub location: Option<String>,
}
impl PersistablePeerDescriptor {
    pub fn insert(&self, id: Id, connection: &mut PgConnection) -> PersistenceResult<()> {
        diesel::insert_into(schema::peer_descriptor::table)
            .values(self)
            .on_conflict(schema::peer_descriptor::peer_id)
            .do_update()
            .set(self)
            .execute(connection)
            .map_err(|cause| PersistenceError::insert::<PeerDescriptor>(id, cause))?;
        Ok(())
    }

    pub fn get(id: Id, connection: &mut PgConnection) -> PersistenceResult<Option<Self>> {
        schema::peer_descriptor::table
            .filter(schema::peer_descriptor::peer_id.eq(id.value()))
            .select(PersistablePeerDescriptor::as_select())
            .first(connection)
            .optional()
            .map_err(|cause| PersistenceError::get::<PeerDescriptor>(id, cause))
    }

    pub fn list(connection: &mut PgConnection) -> PersistenceResult<Vec<Self>> {
        schema::peer_descriptor::table
            .select(PersistablePeerDescriptor::as_select())
            .get_results(connection)
            .map_err(PersistenceError::list::<PeerDescriptor>)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::persistence::database;

    #[tokio::test]
    async fn should_persist_peer_descriptor_base_model() -> anyhow::Result<()> {
        let mut db = database::testing::spawn_and_connect().await?;

        let peer_id = PeerId::random();
        let testee = PersistablePeerDescriptor {
            peer_id: peer_id.uuid,
            name: String::from("testee"),
            location: None,
        };

        let result = PersistablePeerDescriptor::get(peer_id.into(), &mut db.connection)?;
        assert!(result.is_none());
        let result = PersistablePeerDescriptor::list(&mut db.connection)?;
        assert!(result.is_empty());

        testee.insert(peer_id.into(), &mut db.connection)?;

        let result = PersistablePeerDescriptor::get(peer_id.into(), &mut db.connection)?;
        assert_eq!(result, Some(testee.clone()));
        let result = PersistablePeerDescriptor::list(&mut db.connection)?;
        assert_eq!(result.len(), 1);
        assert_eq!(result.first(), Some(&testee));

        Ok(())
    }
}
