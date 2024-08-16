use crate::persistence::database::schema;
use crate::persistence::error::{PersistenceError, PersistenceResult};
use crate::persistence::model::query::Filter;
use diesel::{ExpressionMethods, PgConnection, QueryDsl, QueryResult, RunQueryDsl};
use uuid::Uuid;
use opendut_types::cluster::{ClusterDeployment, ClusterId};

pub fn insert(cluster_id: ClusterId, connection: &mut PgConnection) -> PersistenceResult<()> {
    set_deployment_requested(cluster_id, true, connection)
        .map_err(|cause| PersistenceError::insert::<ClusterDeployment>(cluster_id.0, cause))?;
    Ok(())
}

pub fn remove(cluster_id: ClusterId, connection: &mut PgConnection) -> PersistenceResult<Option<ClusterDeployment>> {
    let result = list(Filter::By(cluster_id), connection)?
        .first().cloned();

    set_deployment_requested(cluster_id, false, connection)
        .map_err(|cause| PersistenceError::remove::<ClusterDeployment>(cluster_id.0, cause))?;

    Ok(result)
}

fn set_deployment_requested(cluster_id: ClusterId, value: bool, connection: &mut PgConnection) -> QueryResult<usize> {
    diesel::update(schema::cluster_configuration::table)
        .filter(schema::cluster_configuration::cluster_id.eq(cluster_id.0))
        .set(schema::cluster_configuration::deployment_requested.eq(value))
        .execute(connection)
}

pub fn list(filter_by_cluster_id: Filter<ClusterId>, connection: &mut PgConnection) -> PersistenceResult<Vec<ClusterDeployment>> {
    let cluster_deployment_ids: Vec<Uuid> = {
        let mut query = schema::cluster_configuration::table.into_boxed();

        if let Filter::By(cluster_id) = filter_by_cluster_id {
            query = query.filter(schema::cluster_configuration::cluster_id.eq(cluster_id.0));
        }

        query
            .filter(schema::cluster_configuration::deployment_requested.eq(true))
            .select(schema::cluster_configuration::cluster_id)
            .get_results(connection)
            .map_err(PersistenceError::list::<ClusterDeployment>)?
    };


    cluster_deployment_ids.into_iter().map(|cluster_id| {
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
