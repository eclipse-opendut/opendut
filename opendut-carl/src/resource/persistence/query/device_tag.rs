use crate::resource::persistence::database::schema;
use crate::resource::persistence::error::{PersistenceError, PersistenceResult};
use diesel::{PgConnection, RunQueryDsl};
use opendut_types::topology::DeviceTag;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, diesel::Queryable, diesel::Selectable, diesel::Insertable, diesel::AsChangeset)]
#[diesel(table_name = schema::device_tag)]
#[diesel(belongs_to(DeviceDescriptor, foreign_key = device_id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PersistableDeviceTag {
    pub device_id: Uuid,
    pub name: String,
}
pub fn insert(persistable: PersistableDeviceTag, connection: &mut PgConnection) -> PersistenceResult<()> {
    diesel::insert_into(schema::device_tag::table)
        .values(&persistable)
        .on_conflict((schema::device_tag::device_id, schema::device_tag::name))
        .do_update()
        .set(&persistable)
        .execute(connection)
        .map_err(|cause| PersistenceError::insert::<DeviceTag>(persistable.device_id, cause))?;
    Ok(())
}

pub fn device_tag_from_persistable(persistable: PersistableDeviceTag) -> PersistenceResult<DeviceTag> {
    let PersistableDeviceTag { device_id, name } = persistable;

    let result = DeviceTag::try_from(name)
        .map_err(|cause| PersistenceError::get::<DeviceTag>(device_id, cause))?;

    Ok(result)
}
