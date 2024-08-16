use diesel::{Connection, PgConnection};
use opendut_types::cluster::{ClusterDeployment, ClusterId};
use std::ops::DerefMut;

use crate::persistence::error::{PersistenceError, PersistenceResult};
use crate::persistence::model::query::cluster_deployment::PersistableClusterDeployment;
use crate::persistence::model::query::Filter;
use crate::persistence::Storage;

use super::{query, Persistable};

impl Persistable for ClusterDeployment {
    fn insert(self, _id: ClusterId, storage: &mut Storage) -> PersistenceResult<()> {
        storage.db.lock().unwrap().transaction::<_, PersistenceError, _>(|connection| {
            insert_into_database(self, connection)
        })
    }

    fn remove(id: ClusterId, storage: &mut Storage) -> PersistenceResult<Option<Self>> {
        remove_from_database(id, storage.db.lock().unwrap().deref_mut())
    }

    fn get(id: ClusterId, storage: &Storage) -> PersistenceResult<Option<Self>> {
        get_from_database(id, storage.db.lock().unwrap().deref_mut())
    }

    fn list(storage: &Storage) -> PersistenceResult<Vec<Self>> {
        list_database(storage.db.lock().unwrap().deref_mut())
    }
}


fn insert_into_database(cluster_deployment: ClusterDeployment, connection: &mut PgConnection) -> PersistenceResult<()> {
    let ClusterDeployment { id } = cluster_deployment;

    query::cluster_deployment::insert(PersistableClusterDeployment {
        cluster_id: id.0,
    }, connection)?;

    Ok(())
}

fn remove_from_database(cluster_id: ClusterId, connection: &mut PgConnection) -> PersistenceResult<Option<ClusterDeployment>> {
    query::cluster_deployment::remove(cluster_id, connection)
}

fn get_from_database(cluster_id: ClusterId, connection: &mut PgConnection) -> PersistenceResult<Option<ClusterDeployment>> {
    let result = query::cluster_deployment::list(Filter::By(cluster_id), connection)?
        .first().cloned();
    Ok(result)
}

fn list_database(connection: &mut PgConnection) -> PersistenceResult<Vec<ClusterDeployment>> {
    query::cluster_deployment::list(Filter::Not, connection)
}


#[cfg(test)]
pub(super) mod tests {
    use super::*;
    use crate::persistence::database;

    #[tokio::test]
    async fn should_persist_cluster_deployment() -> anyhow::Result<()> {
        let mut db = database::testing::spawn_and_connect().await?;

        let peer_descriptor = crate::persistence::model::peer_descriptor::tests::peer_descriptor()?;
        crate::persistence::model::peer_descriptor::insert_into_database(peer_descriptor.clone(), &mut db.connection)?;

        let cluster_configuration = crate::persistence::model::cluster_configuration::tests::cluster_configuration(
            peer_descriptor.id,
            peer_descriptor.topology.devices.into_iter().map(|device| device.id).collect()
        )?;
        crate::persistence::model::cluster_configuration::insert_into_database(cluster_configuration.clone(), &mut db.connection)?;

        let testee = ClusterDeployment {
            id: cluster_configuration.id,
        };

        let result = get_from_database(testee.id, &mut db.connection)?;
        assert!(result.is_none());
        let result = list_database(&mut db.connection)?;
        assert!(result.is_empty());

        insert_into_database(testee.clone(), &mut db.connection)?;

        let result = get_from_database(testee.id, &mut db.connection)?;
        assert_eq!(result, Some(testee.clone()));
        let result = list_database(&mut db.connection)?;
        assert_eq!(result.len(), 1);
        assert_eq!(result.first(), Some(&testee));

        let result = remove_from_database(testee.id, &mut db.connection)?;
        assert_eq!(result, Some(testee.clone()));

        let result = get_from_database(testee.id, &mut db.connection)?;
        assert!(result.is_none());
        let result = list_database(&mut db.connection)?;
        assert!(result.is_empty());

        let result = remove_from_database(testee.id, &mut db.connection)?;
        assert_eq!(result, None);

        Ok(())
    }
}
