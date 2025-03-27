use opendut_types::cluster::{ClusterDeployment, ClusterId};
use std::collections::HashMap;
use redb::TableDefinition;
use crate::resource::api::id::ResourceId;
use crate::resource::persistence;
use crate::resource::persistence::error::PersistenceResult;
use crate::resource::persistence::{Db, Memory};

use super::Persistable;

impl Persistable for ClusterDeployment {
    fn insert(self, cluster_id: ClusterId, _: &mut Memory, db: &Db) -> PersistenceResult<()> {
        let key = persistence::Key::from(ResourceId::<Self>::into_id(cluster_id));

        let value = self;
        let value = serde_json::to_string(&value).unwrap(); //TODO don't unwrap

        let mut table = db.read_write_table(CLUSTER_DEPLOYMENT_TABLE)?;
        table.insert(key, value).unwrap(); //TODO don't unwrap

        Ok(())
    }

    fn remove(cluster_id: ClusterId, _: &mut Memory, db: &Db) -> PersistenceResult<Option<Self>> {
        let key = persistence::Key::from(ResourceId::<Self>::into_id(cluster_id));

        let mut table = db.read_write_table(CLUSTER_DEPLOYMENT_TABLE)?;

        let value = table.remove(key).unwrap() //TODO don't unwrap
            .map(|value| {
                serde_json::from_str::<ClusterDeployment>(&value.value()).unwrap() //TODO don't unwrap
            });

        Ok(value)
    }

    fn get(cluster_id: ClusterId, _: &Memory, db: &Db) -> PersistenceResult<Option<Self>> {
        let key = persistence::Key::from(ResourceId::<Self>::into_id(cluster_id));

        let value = db.read_table(CLUSTER_DEPLOYMENT_TABLE)?
            .and_then(|table| {
                table.get(&key).unwrap() //TODO don't unwrap
                    .map(|value| {
                        serde_json::from_str::<ClusterDeployment>(&value.value()).unwrap() //TODO don't unwrap
                    })
            });

        Ok(value)
    }

    fn list(_: &Memory, db: &Db) -> PersistenceResult<HashMap<Self::Id, Self>> {
        let values = db.read_table(CLUSTER_DEPLOYMENT_TABLE)?
            .map(|table| {
                table.iter().unwrap() //TODO don't unwrap
                    .map(|value| {
                        let (key, value) = value.unwrap(); //TODO don't unwrap
                        let id: ClusterId = ResourceId::<ClusterDeployment>::from_id(key.value().id);

                        let value = serde_json::from_str::<ClusterDeployment>(&value.value()).unwrap(); //TODO don't unwrap

                        (id, value)
                    })
                    .collect::<HashMap<_, _>>()
            })
            .unwrap_or_default();

        Ok(values)
    }
}

const CLUSTER_DEPLOYMENT_TABLE: persistence::TableDefinition = TableDefinition::new("cluster_deployment");

//TODO SerializableClusterDeployment
