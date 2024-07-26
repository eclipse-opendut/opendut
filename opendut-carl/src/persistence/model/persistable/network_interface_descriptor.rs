use diesel::{ExpressionMethods, OptionalExtension, PgConnection, QueryDsl, RunQueryDsl, SelectableHelper};
use uuid::Uuid;

use opendut_types::util::net::{NetworkInterfaceDescriptor, NetworkInterfaceId};

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
    pub fn insert(&self, network_interface_id: NetworkInterfaceId, connection: &mut PgConnection) -> PersistenceResult<()> {
        diesel::insert_into(schema::network_interface::table)
            .values(self)
            .on_conflict(schema::network_interface::network_interface_id)
            .do_update()
            .set(self)
            .execute(connection)
            .map_err(|cause| PersistenceError::insert::<NetworkInterfaceDescriptor>(network_interface_id.uuid, cause))?;
        Ok(())
    }

    pub fn get(network_interface_id: NetworkInterfaceId, connection: &mut PgConnection) -> PersistenceResult<Option<Self>> {
        schema::network_interface::table
            .filter(schema::network_interface::network_interface_id.eq(network_interface_id.uuid))
            .select(Self::as_select())
            .first(connection)
            .optional()
            .map_err(|cause| PersistenceError::get::<NetworkInterfaceDescriptor>(network_interface_id.uuid, cause))
    }

    pub fn list(connection: &mut PgConnection) -> PersistenceResult<Vec<Self>> {
        schema::network_interface::table
            .select(Self::as_select())
            .get_results(connection)
            .map_err(PersistenceError::list::<NetworkInterfaceDescriptor>)
    }
}
