use diesel::{Connection, ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl, SelectableHelper};
use uuid::Uuid;

use opendut_types::peer::PeerId;
use opendut_types::util::net::{NetworkInterfaceDescriptor, NetworkInterfaceId};

use crate::persistence::database::schema;
use crate::persistence::error::{PersistenceError, PersistenceResult};
use crate::persistence::model::persistable::network_interface_kind::PersistableNetworkInterfaceKind;

#[derive(diesel::Queryable, diesel::Selectable, diesel::Insertable, diesel::AsChangeset)]
#[diesel(table_name = schema::network_interface_descriptor)]
#[diesel(belongs_to(PeerDescriptor, foreign_key = peer_id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PersistableNetworkInterfaceDescriptor {
    pub network_interface_id: Uuid,
    pub name: String,
    pub kind: PersistableNetworkInterfaceKind,
    pub peer_id: Uuid,
}

#[derive(diesel::Queryable, diesel::Selectable, diesel::Insertable, diesel::Identifiable, diesel::Associations, diesel::AsChangeset, Debug, PartialEq)]
#[diesel(table_name = schema::network_interface_kind_can)]
#[diesel(primary_key(network_interface_id))]
#[diesel(belongs_to(PersistableNetworkInterfaceDescriptor, foreign_key = network_interface_id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PersistableNetworkInterfaceKindCan {
    pub network_interface_id: Uuid,
    pub bitrate: i32,
    pub sample_point_times_1000: i32,
    pub fd: bool,
    pub data_bitrate: i32,
    pub data_sample_point_times_1000: i32,
}

pub fn insert(
    network_interface_descriptor: PersistableNetworkInterfaceDescriptor,
    maybe_network_interface_kind_can: Option<PersistableNetworkInterfaceKindCan>,
    network_interface_id: NetworkInterfaceId,
    connection: &mut PgConnection
) -> PersistenceResult<()> {

    connection.transaction::<_, PersistenceError, _>(|connection| {

        diesel::insert_into(schema::network_interface_descriptor::table)
            .values(&network_interface_descriptor)
            .on_conflict(schema::network_interface_descriptor::network_interface_id)
            .do_update()
            .set(&network_interface_descriptor)
            .execute(connection)
            .map_err(|cause| PersistenceError::insert::<NetworkInterfaceDescriptor>(network_interface_id.uuid, cause))?;

        maybe_network_interface_kind_can.map(|network_interface_kind_can| {
            diesel::insert_into(schema::network_interface_kind_can::table)
                .values(&network_interface_kind_can)
                .on_conflict(schema::network_interface_kind_can::network_interface_id)
                .do_update()
                .set(&network_interface_kind_can)
                .execute(connection)
                .map_err(|cause| PersistenceError::insert::<PersistableNetworkInterfaceKindCan>(network_interface_id.uuid, cause))
        }).transpose()?;

        Ok(())
    })?;

    Ok(())
}

pub fn list_filtered_by_peer_id(
    peer_id: PeerId,
    connection: &mut PgConnection
) -> PersistenceResult<Vec<(
    PersistableNetworkInterfaceDescriptor,
    Option<PersistableNetworkInterfaceKindCan>
)>> {
    schema::network_interface_descriptor::table
        .left_join(schema::network_interface_kind_can::table)
        .filter(schema::network_interface_descriptor::peer_id.eq(peer_id.uuid))
        .select((PersistableNetworkInterfaceDescriptor::as_select(), Option::<PersistableNetworkInterfaceKindCan>::as_select()))
        .get_results(connection)
        .map_err(PersistenceError::list::<NetworkInterfaceDescriptor>)
}
