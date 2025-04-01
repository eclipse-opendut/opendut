use crate::resource::api::id::ResourceId;
use crate::resource::persistence;
use crate::resource::persistence::error::PersistenceResult;
use crate::resource::persistence::{Db, Memory};
use opendut_types::cluster::{ClusterDeployment, ClusterId};
use persistence::TableDefinition;
use std::collections::HashMap;

use super::Persistable;

impl Persistable for ClusterDeployment {
    fn insert(self, cluster_id: ClusterId, _: &mut Memory, db: &Db) -> PersistenceResult<()> {
        let key = persistence::Key::from(ResourceId::<Self>::into_id(cluster_id));

        let value = self;
        let value = serde_json::to_string(&value)?;

        let mut table = db.read_write_table(CLUSTER_DEPLOYMENT_TABLE)?;
        table.insert(key, value)?;

        Ok(())
    }

    fn remove(cluster_id: ClusterId, _: &mut Memory, db: &Db) -> PersistenceResult<Option<Self>> {
        let key = persistence::Key::from(ResourceId::<Self>::into_id(cluster_id));

        let mut table = db.read_write_table(CLUSTER_DEPLOYMENT_TABLE)?;

        let value = table.remove(key)?
            .map(|value| {
                serde_json::from_str::<ClusterDeployment>(&value.value())
            })
            .transpose()?;

        Ok(value)
    }

    fn get(cluster_id: ClusterId, _: &Memory, db: &Db) -> PersistenceResult<Option<Self>> {
        let key = persistence::Key::from(ResourceId::<Self>::into_id(cluster_id));

        if let Some(table) = db.read_table(CLUSTER_DEPLOYMENT_TABLE)? {
            let value = table.get(&key)?
                .map(|value| {
                    serde_json::from_str::<ClusterDeployment>(&value.value())
                })
                .transpose()?;
            Ok(value)
        } else {
            Ok(None)
        }
    }

    fn list(_: &Memory, db: &Db) -> PersistenceResult<HashMap<Self::Id, Self>> {
        if let Some(table) = db.read_table(CLUSTER_DEPLOYMENT_TABLE)? {
            table.iter()?
                .map(|value| {
                    let (key, value) = value?;
                    let id: ClusterId = ResourceId::<ClusterDeployment>::from_id(key.value().id);

                    let value = serde_json::from_str::<ClusterDeployment>(&value.value())?;

                    Ok((id, value))
                })
                .collect::<PersistenceResult<HashMap<_, _>>>()
        } else {
            Ok(HashMap::default())
        }
    }
}

const CLUSTER_DEPLOYMENT_TABLE: TableDefinition = TableDefinition::new("cluster_deployment");

//TODO SerializableClusterDeployment
