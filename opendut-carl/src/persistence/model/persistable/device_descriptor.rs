use diesel::{ExpressionMethods, OptionalExtension, PgConnection, QueryDsl, RunQueryDsl, SelectableHelper};
use uuid::Uuid;
use opendut_types::peer::PeerId;
use opendut_types::topology::{DeviceDescription, DeviceDescriptor, DeviceId, DeviceName};
use opendut_types::util::net::NetworkInterfaceId;
use crate::persistence::database::schema;
use crate::persistence::error::{PersistenceError, PersistenceResult};

#[derive(Clone, Debug, PartialEq, diesel::Queryable, diesel::Selectable, diesel::Insertable, diesel::AsChangeset)]
#[diesel(table_name = schema::device_descriptor)]
#[diesel(belongs_to(NetworkInterfaceDescriptor, foreign_key = network_interface_id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PersistableDeviceDescriptor {
    pub device_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub network_interface_id: Option<Uuid>,
}
impl PersistableDeviceDescriptor {
    pub fn insert(&self, device_id: DeviceId, connection: &mut PgConnection) -> PersistenceResult<()> {
        diesel::insert_into(schema::device_descriptor::table)
            .values(self)
            .on_conflict(schema::device_descriptor::device_id)
            .do_update()
            .set(self)
            .execute(connection)
            .map_err(|cause| PersistenceError::insert::<DeviceDescriptor>(device_id.0, cause))?;
        Ok(())
    }

    pub fn get(device_id: DeviceId, connection: &mut PgConnection) -> PersistenceResult<Option<Self>> {
        schema::device_descriptor::table
            .filter(schema::device_descriptor::device_id.eq(device_id.0))
            .select(Self::as_select())
            .first(connection)
            .optional()
            .map_err(|cause| PersistenceError::get::<DeviceDescriptor>(device_id.0, cause))
    }

    pub fn list(connection: &mut PgConnection) -> PersistenceResult<Vec<Self>> {
        schema::device_descriptor::table
            .select(Self::as_select())
            .get_results(connection)
            .map_err(PersistenceError::list::<DeviceDescriptor>)
    }

    pub fn list_filtered_by_peer(peer_id: PeerId, connection: &mut PgConnection) -> PersistenceResult<Vec<Self>> {
        schema::device_descriptor::table
            .left_join(schema::network_interface_descriptor::table)
            .filter(schema::network_interface_descriptor::peer_id.eq(peer_id.uuid))
            .select(Self::as_select())
            .get_results(connection)
            .map_err(PersistenceError::list::<DeviceDescriptor>)
    }
}

pub fn device_descriptor_from_persistable(persistable: PersistableDeviceDescriptor) -> PersistenceResult<DeviceDescriptor> {
    let PersistableDeviceDescriptor { device_id, name, description, network_interface_id } = persistable;

    let result = DeviceDescriptor {
        id: DeviceId::from(device_id),
        name: DeviceName::try_from(name)
            .map_err(|cause| PersistenceError::get::<DeviceDescriptor>(device_id, cause))?,
        description: description.map(DeviceDescription::try_from).transpose()
            .map_err(|cause| PersistenceError::get::<DeviceDescriptor>(device_id, cause))?,
        interface: network_interface_id.map(NetworkInterfaceId::from)
            .expect("We should always have a NetworkInterfaceId persisted for now."), //TODO DeviceDescriptor should use an Option<NetworkInterfaceId>
        tags: vec![], //TODO
    };
    Ok(result)
}
