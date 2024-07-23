use diesel::{ExpressionMethods, OptionalExtension, PgConnection, QueryDsl, RunQueryDsl, SelectableHelper};
use uuid::Uuid;

use opendut_types::resources::Id;
use opendut_types::util::net::NetworkInterfaceDescriptor;

use crate::persistence::database::schema;
use crate::persistence::error::{PersistenceError, PersistenceResult};
use crate::persistence::model::persistable::network_interface_kind::PersistableNetworkInterfaceKind;

#[derive(diesel::Queryable, diesel::Selectable, diesel::Insertable, diesel::AsChangeset)]
#[diesel(table_name = schema::network_interface)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PersistableNetworkInterfaceDescriptor {
    pub network_interface_id: Uuid,
    pub name: String,
    pub kind: PersistableNetworkInterfaceKind,
    pub peer_id: Uuid,
}
impl PersistableNetworkInterfaceDescriptor {
    pub fn insert(&self, id: Id, connection: &mut PgConnection) -> PersistenceResult<()> {
        diesel::insert_into(schema::network_interface::table)
            .values(self)
            .on_conflict(schema::network_interface::network_interface_id)
            .do_update()
            .set(self)
            .execute(connection)
            .map_err(|cause| PersistenceError::insert::<NetworkInterfaceDescriptor>(id, cause))?;
        Ok(())
    }

    pub fn get(id: Id, connection: &mut PgConnection) -> PersistenceResult<Option<Self>> {
        schema::network_interface::table
            .filter(schema::network_interface::network_interface_id.eq(id.value()))
            .select(Self::as_select())
            .first(connection)
            .optional()
            .map_err(|cause| PersistenceError::get::<NetworkInterfaceDescriptor>(id, cause))
    }

    pub fn list(connection: &mut PgConnection) -> PersistenceResult<Vec<Self>> {
        schema::network_interface::table
            .select(Self::as_select())
            .get_results(connection)
            .map_err(PersistenceError::list::<NetworkInterfaceDescriptor>)
    }
}
