use crate::persistence::database::schema;
use crate::persistence::error::{PersistenceError, PersistenceResult};
use crate::persistence::model::query::Filter;
use diesel::{ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl, SelectableHelper};
use opendut_types::cluster::{ClusterDeployment, ClusterId};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, diesel::Queryable, diesel::Selectable, diesel::Insertable, diesel::AsChangeset)]
#[diesel(table_name = schema::cluster_deployment)]
#[diesel(belongs_to(PersistableClusterConfiguration, foreign_key = cluster_id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PersistableClusterDeployment {
    pub cluster_id: Uuid,
}
pub fn insert(persistable: PersistableClusterDeployment, connection: &mut PgConnection) -> PersistenceResult<()> {
    diesel::insert_into(schema::cluster_deployment::table)
        .values(&persistable)
        .on_conflict(schema::cluster_deployment::cluster_id)
        .do_update()
        .set(&persistable)
        .execute(connection)
        .map_err(|cause| PersistenceError::insert::<ClusterDeployment>(persistable.cluster_id, cause))?;
    Ok(())
}

pub fn remove(cluster_id: ClusterId, connection: &mut PgConnection) -> PersistenceResult<Option<ClusterDeployment>> {
    let result = list(Filter::By(cluster_id), connection)?
        .first().cloned();

    diesel::delete(
        schema::cluster_deployment::table
            .filter(schema::cluster_deployment::cluster_id.eq(cluster_id.0))
    )
    .execute(connection)
    .map_err(|cause| PersistenceError::remove::<ClusterDeployment>(cluster_id.0, cause))?;

    Ok(result)
}

pub fn list(filter_by_cluster_id: Filter<ClusterId>, connection: &mut PgConnection) -> PersistenceResult<Vec<ClusterDeployment>> {
    let persistable_cluster_deployments = {
        let mut query = schema::cluster_deployment::table.into_boxed();

        if let Filter::By(cluster_id) = filter_by_cluster_id {
            query = query.filter(schema::cluster_deployment::cluster_id.eq(cluster_id.0));
        }

        query
            .select(PersistableClusterDeployment::as_select())
            .get_results(connection)
            .map_err(PersistenceError::list::<ClusterDeployment>)?
    };


    persistable_cluster_deployments.into_iter().map(|persistable| {
        let PersistableClusterDeployment { cluster_id } = persistable;

        let cluster_id = ClusterId::from(cluster_id);

        Ok(ClusterDeployment {
            id: cluster_id,
        })
    })
    .collect::<PersistenceResult<Vec<_>>>()
    .map_err(|cause|
        PersistenceError::list::<ClusterDeployment>(cause)
            .context("Failed to convert from database values to ClusterDeployment.")
    )
}
