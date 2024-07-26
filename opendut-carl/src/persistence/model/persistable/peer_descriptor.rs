use diesel::{ExpressionMethods, OptionalExtension, PgConnection, QueryDsl, RunQueryDsl, SelectableHelper};
use uuid::Uuid;

use opendut_types::peer::{PeerDescriptor, PeerId};

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
    pub fn insert(&self, peer_id: PeerId, connection: &mut PgConnection) -> PersistenceResult<()> {
        diesel::insert_into(schema::peer_descriptor::table)
            .values(self)
            .on_conflict(schema::peer_descriptor::peer_id)
            .do_update()
            .set(self)
            .execute(connection)
            .map_err(|cause| PersistenceError::insert::<PeerDescriptor>(peer_id.uuid, cause))?;
        Ok(())
    }

    pub fn get(peer_id: PeerId, connection: &mut PgConnection) -> PersistenceResult<Option<Self>> {
        schema::peer_descriptor::table
            .filter(schema::peer_descriptor::peer_id.eq(peer_id.uuid))
            .select(Self::as_select())
            .first(connection)
            .optional()
            .map_err(|cause| PersistenceError::get::<PeerDescriptor>(peer_id.uuid, cause))
    }

    pub fn list(connection: &mut PgConnection) -> PersistenceResult<Vec<Self>> {
        schema::peer_descriptor::table
            .select(Self::as_select())
            .get_results(connection)
            .map_err(PersistenceError::list::<PeerDescriptor>)
    }
}
