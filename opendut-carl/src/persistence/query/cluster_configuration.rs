use crate::persistence::database::schema;
use crate::persistence::error::{PersistenceError, PersistenceResult};
use crate::persistence::query;
use crate::persistence::query::cluster_device::PersistableClusterDevice;
use crate::persistence::query::Filter;
use diesel::{ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl, SelectableHelper};
use opendut_types::cluster::{ClusterConfiguration, ClusterId, ClusterName};
use opendut_types::peer::PeerId;
use opendut_types::topology::DeviceId;
use std::collections::HashSet;
use std::ops::Not;
use uuid::Uuid;

pub fn insert(cluster_configuration: ClusterConfiguration, connection: &mut PgConnection) -> PersistenceResult<()> {
    let ClusterConfiguration { id, name, leader, devices } = cluster_configuration;

    insert_persistable(PersistableClusterConfiguration {
        cluster_id: id.0,
        name: name.value(),
        leader_id: leader.uuid,
    }, connection)?;

    {
        let previous_cluster_devices = query::cluster_device::list_filtered_by_cluster_id(id, connection)?;

        for previous_device in previous_cluster_devices {
            if devices.contains(&DeviceId::from(previous_device.device_id)).not() {
                query::cluster_device::remove(previous_device, connection)?;
            }
        }

        for device in devices {
            query::cluster_device::insert(PersistableClusterDevice {
                cluster_id: id.0,
                device_id: device.0,
            }, connection)?
        }
    }

    Ok(())
}

#[derive(Clone, Debug, PartialEq, diesel::Queryable, diesel::Selectable, diesel::Insertable, diesel::AsChangeset)]
#[diesel(table_name = schema::cluster_configuration)]
#[diesel(belongs_to(PersistablePeerDescriptor, foreign_key = leader_id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct PersistableClusterConfiguration {
    pub cluster_id: Uuid,
    pub name: String,
    pub leader_id: Uuid,
}
fn insert_persistable(persistable: PersistableClusterConfiguration, connection: &mut PgConnection) -> PersistenceResult<()> {
    diesel::insert_into(schema::cluster_configuration::table)
        .values(&persistable)
        .on_conflict(schema::cluster_configuration::cluster_id)
        .do_update()
        .set(&persistable)
        .execute(connection)
        .map_err(|cause| PersistenceError::insert::<ClusterConfiguration>(persistable.cluster_id, cause))?;
    Ok(())
}

pub fn remove(cluster_id: ClusterId, connection: &mut PgConnection) -> PersistenceResult<Option<ClusterConfiguration>> {
    let result = list(Filter::By(cluster_id), connection)?
        .first().cloned();

    diesel::delete(
        schema::cluster_configuration::table
            .filter(schema::cluster_configuration::cluster_id.eq(cluster_id.0))
    )
    .execute(connection)
    .map_err(|cause| PersistenceError::remove::<ClusterConfiguration>(cluster_id.0, cause))?;

    Ok(result)
}

pub fn list(filter_by_cluster_id: Filter<ClusterId>, connection: &mut PgConnection) -> PersistenceResult<Vec<ClusterConfiguration>> {
    let persistable_cluster_configurations = {
        let mut query = schema::cluster_configuration::table.into_boxed();

        if let Filter::By(cluster_id) = filter_by_cluster_id {
            query = query.filter(schema::cluster_configuration::cluster_id.eq(cluster_id.0));
        }

        query
            .select(PersistableClusterConfiguration::as_select())
            .get_results(connection)
            .map_err(PersistenceError::list::<ClusterConfiguration>)?
    };


    persistable_cluster_configurations.into_iter().map(|persistable| {
        let PersistableClusterConfiguration { cluster_id, name, leader_id } = persistable;

        let cluster_id = ClusterId::from(cluster_id);

        let name = ClusterName::try_from(name)
            .map_err(|cause| PersistenceError::get::<ClusterConfiguration>(cluster_id.0, cause))?;

        let leader_id = PeerId::from(leader_id);

        let devices = query::cluster_device::list_filtered_by_cluster_id(cluster_id, connection)?
            .into_iter()
            .map(|cluster_device| DeviceId::from(cluster_device.device_id))
            .collect::<HashSet<_>>();

        Ok(ClusterConfiguration {
            id: cluster_id,
            name,
            leader: leader_id,
            devices,
        })
    })
    .collect::<PersistenceResult<Vec<_>>>()
    .map_err(|cause|
        PersistenceError::list::<ClusterConfiguration>(cause)
            .context("Failed to convert from database values to ClusterConfiguration.")
    )
}
