use crate::resource::persistence::database::schema;
use crate::resource::persistence::error::{PersistenceError, PersistenceOperation, PersistenceResult};
use crate::resource::persistence::query::Filter;
use diesel::{ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl};
use opendut_types::cluster::{ClusterDeployment, ClusterId};
use std::collections::HashMap;
use uuid::Uuid;

pub fn insert(cluster_deployment: ClusterDeployment, connection: &mut PgConnection) -> PersistenceResult<()> {
    let ClusterDeployment { id } = cluster_deployment;

    insert_persistable(id, connection)?;

    Ok(())
}

fn insert_persistable(cluster_id: ClusterId, connection: &mut PgConnection) -> PersistenceResult<()> {
    let requested = true;
    set_deployment_requested(cluster_id, requested, connection, PersistenceOperation::Insert)
}

pub fn remove(cluster_id: ClusterId, connection: &mut PgConnection) -> PersistenceResult<Option<ClusterDeployment>> {
    let result = list(Filter::By(cluster_id), connection)?.values().next().cloned();

    let requested = false;
    set_deployment_requested(cluster_id, requested, connection, PersistenceOperation::Remove)?;

    Ok(result)
}

fn set_deployment_requested(cluster_id: ClusterId, value: bool, connection: &mut PgConnection, operation: PersistenceOperation) -> PersistenceResult<()> {
    let result = diesel::update(schema::cluster_configuration::table) //TODO error when non-existent?
        .filter(schema::cluster_configuration::cluster_id.eq(cluster_id.0))
        .set(schema::cluster_configuration::deployment_requested.eq(value))
        .execute(connection);

    match result {
        Ok(changed_rows) if changed_rows > 0 => Ok(()),
        Ok(_) => Err(PersistenceError::new::<ClusterDeployment>(Some(cluster_id.0), operation, Some(format!("No database entries found for cluster {cluster_id}.")))),
        Err(source) => Err(PersistenceError::new::<ClusterDeployment>(Some(cluster_id.0), operation, Some(source))),
    }
}

pub fn list(filter_by_cluster_id: Filter<ClusterId>, connection: &mut PgConnection) -> PersistenceResult<HashMap<ClusterId, ClusterDeployment>> {
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

        Ok((cluster_id,
            ClusterDeployment {
                id: cluster_id,
            }
        ))
    })
    .collect::<PersistenceResult<HashMap<_, _>>>()
    .map_err(|cause|
        PersistenceError::list::<ClusterDeployment>(cause)
            .context("Failed to convert from database values to ClusterDeployment.")
    )
}
