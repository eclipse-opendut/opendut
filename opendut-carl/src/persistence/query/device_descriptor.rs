use crate::persistence::database::schema;
use crate::persistence::error::{PersistenceError, PersistenceResult};
use crate::persistence::query;
use crate::persistence::query::device_tag::{device_tag_from_persistable, PersistableDeviceTag};
use crate::persistence::query::Filter;
use diesel::query_builder::AsQuery;
use diesel::{ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl, SelectableHelper};
use opendut_types::peer::PeerId;
use opendut_types::topology::{DeviceDescription, DeviceDescriptor, DeviceId, DeviceName, DeviceTag};
use opendut_types::util::net::NetworkInterfaceId;
use uuid::Uuid;

pub fn insert(device_descriptor: DeviceDescriptor, connection: &mut PgConnection) -> PersistenceResult<()> {
    let DeviceDescriptor { id, name, description, interface, tags } = device_descriptor;

    let name = name.value().to_owned();
    let description = description.map(|description| description.value().to_owned());
    let network_interface_id = Some(interface.uuid);

    insert_persistable(PersistableDeviceDescriptor {
        device_id: id.0,
        name,
        description,
        network_interface_id,
    }, connection)?;

    for tag in tags {
        query::device_tag::insert(PersistableDeviceTag {
            device_id: id.0,
            name: tag.value().to_owned(),
        }, connection)?;
    }

    Ok(())
}


#[derive(Clone, Debug, PartialEq, diesel::Queryable, diesel::Selectable, diesel::Insertable, diesel::AsChangeset)]
#[diesel(table_name = schema::device_descriptor)]
#[diesel(belongs_to(NetworkInterfaceDescriptor, foreign_key = network_interface_id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct PersistableDeviceDescriptor {
    pub device_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub network_interface_id: Option<Uuid>,
}

fn insert_persistable(persistable: PersistableDeviceDescriptor, connection: &mut PgConnection) -> PersistenceResult<()> {
    diesel::insert_into(schema::device_descriptor::table)
        .values(&persistable)
        .on_conflict(schema::device_descriptor::device_id)
        .do_update()
        .set(&persistable)
        .execute(connection)
        .map_err(|cause| PersistenceError::insert::<DeviceDescriptor>(persistable.device_id, cause))?;
    Ok(())
}

pub fn remove(device_id: DeviceId, connection: &mut PgConnection) -> PersistenceResult<Option<DeviceDescriptor>> {
    let result = list(Filter::By(device_id), connection)?
        .first().cloned();

    diesel::delete(
        schema::device_descriptor::table
            .filter(schema::device_descriptor::device_id.eq(device_id.0))
    )
    .execute(connection)
    .map_err(|cause| PersistenceError::remove::<DeviceDescriptor>(device_id.0, cause))?;

    Ok(result)
}

pub fn list(filter_by_device_id: Filter<DeviceId>, connection: &mut PgConnection) -> PersistenceResult<Vec<DeviceDescriptor>> {
    let mut query = schema::device_descriptor::table.into_boxed();

    if let Filter::By(device_id) = filter_by_device_id {
        query = query.filter(schema::device_descriptor::device_id.eq(device_id.0)).as_query()
    }

    query
        .select(PersistableDeviceDescriptor::as_select())
        .get_results(connection)
        .map_err(PersistenceError::list::<DeviceDescriptor>)?
        .into_iter()
        .map(|device| device_descriptor_from_persistable(device, connection))
        .collect::<Result<_, _>>()
}

pub fn list_filtered_by_peer(peer_id: PeerId, connection: &mut PgConnection) -> PersistenceResult<Vec<DeviceDescriptor>> {
    schema::device_descriptor::table
        .left_join(schema::network_interface_descriptor::table)
        .filter(schema::network_interface_descriptor::peer_id.eq(peer_id.uuid))
        .select(PersistableDeviceDescriptor::as_select())
        .get_results(connection)
        .map_err(PersistenceError::list::<DeviceDescriptor>)?
        .into_iter()
        .map(|device| device_descriptor_from_persistable(device, connection))
        .collect::<Result<_, _>>()
}


fn device_descriptor_from_persistable(
    persistable_device_descriptor: PersistableDeviceDescriptor,
    connection: &mut PgConnection,
) -> PersistenceResult<DeviceDescriptor> {
    let PersistableDeviceDescriptor { device_id, name, description, network_interface_id } = persistable_device_descriptor;

    let tags = schema::device_tag::table
        .filter(schema::device_tag::device_id.eq(device_id))
        .select(PersistableDeviceTag::as_select())
        .get_results(connection)
        .map_err(PersistenceError::list::<DeviceTag>)?
        .into_iter()
        .map(device_tag_from_persistable)
        .collect::<Result<_, _>>()?;

    let result = DeviceDescriptor {
        id: DeviceId::from(device_id),
        name: DeviceName::try_from(name)
            .map_err(|cause| PersistenceError::get::<DeviceDescriptor>(device_id, cause))?,
        description: description.map(DeviceDescription::try_from).transpose()
            .map_err(|cause| PersistenceError::get::<DeviceDescriptor>(device_id, cause))?,
        interface: network_interface_id.map(NetworkInterfaceId::from)
            .expect("We should always have a NetworkInterfaceId persisted for now."), //TODO DeviceDescriptor should use an Option<NetworkInterfaceId>
        tags,
    };
    Ok(result)
}
