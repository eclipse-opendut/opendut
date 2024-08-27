use diesel::{Connection, ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl, SelectableHelper};
use uuid::Uuid;

use opendut_types::peer::PeerId;
use opendut_types::util::net::{CanSamplePoint, NetworkInterfaceConfiguration, NetworkInterfaceDescriptor, NetworkInterfaceId, NetworkInterfaceName};

use crate::persistence::database::schema;
use crate::persistence::error::{PersistenceError, PersistenceOperation, PersistenceResult};
use crate::persistence::query::types::network_interface_kind::PersistableNetworkInterfaceKind;

#[derive(diesel::Queryable, diesel::Selectable, diesel::Insertable, diesel::AsChangeset)]
#[diesel(table_name = schema::network_interface_descriptor)]
#[diesel(belongs_to(PeerDescriptor, foreign_key = peer_id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(in crate::persistence) struct PersistableNetworkInterfaceDescriptor {
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
pub(in crate::persistence) struct PersistableNetworkInterfaceKindCan {
    pub network_interface_id: Uuid,
    pub bitrate: i32,
    pub sample_point_times_1000: i32,
    pub fd: bool,
    pub data_bitrate: i32,
    pub data_sample_point_times_1000: i32,
}

pub fn insert(interface: NetworkInterfaceDescriptor, peer_id: PeerId, connection: &mut PgConnection) -> PersistenceResult<()> {
    let network_interface_id = interface.id.uuid;

    let (kind, network_interface_kind_can) = match &interface.configuration {
        NetworkInterfaceConfiguration::Ethernet => {
            (PersistableNetworkInterfaceKind::Ethernet, None)
        }
        NetworkInterfaceConfiguration::Can { bitrate, sample_point, fd, data_bitrate, data_sample_point } => {
            let bitrate = i32::try_from(*bitrate)
                .map_err(|cause| PersistenceError::insert::<PersistableNetworkInterfaceKindCan>(network_interface_id, cause))?;
            let sample_point_times_1000 = i32::try_from(sample_point.sample_point_times_1000())
                .map_err(|cause| PersistenceError::insert::<PersistableNetworkInterfaceKindCan>(network_interface_id, cause))?;
            let data_bitrate = i32::try_from(*data_bitrate)
                .map_err(|cause| PersistenceError::insert::<PersistableNetworkInterfaceKindCan>(network_interface_id, cause))?;
            let data_sample_point_times_1000 = i32::try_from(data_sample_point.sample_point_times_1000())
                .map_err(|cause| PersistenceError::insert::<PersistableNetworkInterfaceKindCan>(network_interface_id, cause))?;

            let network_interface_kind_can = PersistableNetworkInterfaceKindCan {
                network_interface_id,
                bitrate,
                sample_point_times_1000,
                fd: *fd,
                data_bitrate,
                data_sample_point_times_1000,
            };
            (PersistableNetworkInterfaceKind::Can, Some(network_interface_kind_can))
        }
    };
    let network_interface_descriptor = PersistableNetworkInterfaceDescriptor {
        network_interface_id,
        name: interface.name.name(),
        kind,
        peer_id: peer_id.uuid,
    };

    insert_persistable(network_interface_descriptor, network_interface_kind_can, interface.id, connection)
}


fn insert_persistable(
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

pub fn list_filtered_by_peer(
    peer_id: PeerId,
    connection: &mut PgConnection
) -> PersistenceResult<Vec<NetworkInterfaceDescriptor>> {
    let persistables = list_filtered_by_peer_id_persistable(peer_id, connection)?;

    let result = persistables.into_iter().map(|(persistable_network_interface_descriptor, persistable_network_interface_kind_can)| {
        let PersistableNetworkInterfaceDescriptor { network_interface_id, name, kind, peer_id: _ } = persistable_network_interface_descriptor;

        let id = NetworkInterfaceId::from(network_interface_id);
        let name = NetworkInterfaceName::try_from(name)
            .map_err(PersistenceError::list::<NetworkInterfaceDescriptor>)?;

        let configuration = network_interface_configuration_from_persistable(kind, persistable_network_interface_kind_can)?;

        Ok(NetworkInterfaceDescriptor { id, name, configuration })
    }).collect::<PersistenceResult<_>>()?;

    Ok(result)
}

fn list_filtered_by_peer_id_persistable(
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

fn network_interface_configuration_from_persistable(
    persistable_network_interface_kind: PersistableNetworkInterfaceKind,
    persistable_network_interface_kind_can: Option<PersistableNetworkInterfaceKindCan>,
) -> PersistenceResult<NetworkInterfaceConfiguration> {
    let result = match persistable_network_interface_kind {
        PersistableNetworkInterfaceKind::Ethernet => NetworkInterfaceConfiguration::Ethernet,
        PersistableNetworkInterfaceKind::Can => {
            let persistable_network_interface_kind_can = persistable_network_interface_kind_can
                .ok_or(PersistenceError::new::<NetworkInterfaceConfiguration>(None::<Uuid>, PersistenceOperation::List, Option::<PersistenceError>::None))?;

            let PersistableNetworkInterfaceKindCan { network_interface_id: _, bitrate, sample_point_times_1000, fd, data_bitrate, data_sample_point_times_1000 } = persistable_network_interface_kind_can;


            let bitrate = u32::try_from(bitrate)
                .map_err(PersistenceError::list::<NetworkInterfaceConfiguration>)?;

            let sample_point = u32::try_from(sample_point_times_1000)
                .map_err(PersistenceError::list::<NetworkInterfaceConfiguration>)?;
            let sample_point = CanSamplePoint::try_from(sample_point)
                .map_err(PersistenceError::list::<NetworkInterfaceConfiguration>)?;

            let data_bitrate = u32::try_from(data_bitrate)
                .map_err(PersistenceError::list::<NetworkInterfaceConfiguration>)?;

            let data_sample_point = u32::try_from(data_sample_point_times_1000)
                .map_err(PersistenceError::list::<NetworkInterfaceConfiguration>)?;
            let data_sample_point = CanSamplePoint::try_from(data_sample_point)
                .map_err(PersistenceError::list::<NetworkInterfaceConfiguration>)?;

            NetworkInterfaceConfiguration::Can {
                bitrate,
                sample_point,
                fd,
                data_bitrate,
                data_sample_point,
            }
        }
    };
    Ok(result)
}
