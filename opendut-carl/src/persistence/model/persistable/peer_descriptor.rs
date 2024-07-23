use diesel::{ExpressionMethods, OptionalExtension, PgConnection, QueryDsl, RunQueryDsl, SelectableHelper};
use uuid::Uuid;

use opendut_types::peer::PeerDescriptor;
use opendut_types::resources::Id;

use crate::persistence::database::schema;
use crate::persistence::error::{PersistenceError, PersistenceResult};

#[derive(Clone, Debug, PartialEq, diesel::Queryable, diesel::Selectable, diesel::Insertable, diesel::AsChangeset)]
#[diesel(table_name = schema::peer_descriptor)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PersistablePeerDescriptor {
    pub peer_id: Uuid,
    pub name: String,
    pub location: Option<String>,
    pub network_bridge_name: Option<String>,
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
            .select(Self::as_select())
            .first(connection)
            .optional()
            .map_err(|cause| PersistenceError::get::<PeerDescriptor>(id, cause))
    }

    pub fn list(connection: &mut PgConnection) -> PersistenceResult<Vec<Self>> {
        schema::peer_descriptor::table
            .select(Self::as_select())
            .get_results(connection)
            .map_err(PersistenceError::list::<PeerDescriptor>)
    }
}

#[cfg(test)]
mod tests {
    use opendut_types::peer::PeerId;

    use crate::persistence::database;

    use super::*;

    #[tokio::test]
    async fn should_persist_peer_descriptor_base_model() -> anyhow::Result<()> {
        let mut db = database::testing::spawn_and_connect().await?;

        let peer_id = PeerId::random();
        let testee = PersistablePeerDescriptor {
            peer_id: peer_id.0,
            name: String::from("testee"),
            location: None,
            network_bridge_name: None,
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
