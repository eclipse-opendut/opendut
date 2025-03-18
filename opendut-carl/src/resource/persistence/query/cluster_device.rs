use crate::resource::persistence::database::schema;
use crate::resource::persistence::error::{PersistenceError, PersistenceResult};
use diesel::{ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl, SelectableHelper};
use opendut_types::cluster::ClusterId;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, diesel::Queryable, diesel::Selectable, diesel::Insertable, diesel::AsChangeset)]
#[diesel(table_name = schema::cluster_device)]
#[diesel(belongs_to(PersistableClusterConfiguration, foreign_key = cluster_id))]
#[diesel(belongs_to(PersistableDeviceDescriptor, foreign_key = device_id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PersistableClusterDevice {
    pub cluster_id: Uuid,
    pub device_id: Uuid,
}
pub fn insert(persistable: PersistableClusterDevice, connection: &mut PgConnection) -> PersistenceResult<()> {
    diesel::insert_into(schema::cluster_device::table)
        .values(&persistable)
        .on_conflict((schema::cluster_device::cluster_id, schema::cluster_device::device_id))
        .do_update()
        .set(&persistable)
        .execute(connection)
        .map_err(|cause| PersistenceError::insert::<PersistableClusterDevice>(persistable.device_id, cause))?;
    Ok(())
}

pub fn list_filtered_by_cluster_id(cluster_id: ClusterId, connection: &mut PgConnection) -> PersistenceResult<Vec<PersistableClusterDevice>> {
    schema::cluster_device::table
        .filter(schema::cluster_device::cluster_id.eq(cluster_id.0))
        .select(PersistableClusterDevice::as_select())
        .get_results(connection)
        .map_err(PersistenceError::list::<PersistableClusterDevice>)
}

pub fn remove(cluster_device: PersistableClusterDevice, connection: &mut PgConnection) -> PersistenceResult<()> {
    diesel::delete(
        schema::cluster_device::table
            .filter(schema::cluster_device::device_id.eq(cluster_device.device_id))
            .filter(schema::cluster_device::cluster_id.eq(cluster_device.cluster_id))
    )
    .execute(connection)
    .map_err(|cause| PersistenceError::remove::<PersistableClusterDevice>(cluster_device, cause))?;

    Ok(())
}
