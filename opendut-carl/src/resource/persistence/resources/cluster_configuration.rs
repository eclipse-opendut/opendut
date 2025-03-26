use super::Persistable;
use crate::resource::persistence::error::PersistenceResult;
use crate::resource::persistence::Storage;
use opendut_types::cluster::{ClusterConfiguration, ClusterId};
use std::collections::HashMap;
use redb::{ReadableTable, TableDefinition};
use uuid::Uuid;

impl Persistable for ClusterConfiguration {
    fn insert(self, cluster_id: ClusterId, storage: &mut Storage) -> PersistenceResult<()> {

        let key = cluster_id.0.as_bytes().as_slice();

        let value = self;
        let value = serde_json::to_string(&value).unwrap(); //TODO don't unwrap

        let mut table = storage.db.open_table(CLUSTER_CONFIGURATION_TABLE).unwrap(); //TODO don't unwrap
        table.insert(key, value).unwrap(); //TODO don't unwrap

        Ok(())
    }

    fn remove(cluster_id: ClusterId, storage: &mut Storage) -> PersistenceResult<Option<Self>> {
        let mut table = storage.db.open_table(CLUSTER_CONFIGURATION_TABLE).unwrap(); //TODO don't unwrap

        let key = cluster_id.0.as_bytes().as_slice();

        let value = table.remove(key).unwrap() //TODO don't unwrap
            .map(|value| {
                serde_json::from_str::<ClusterConfiguration>(&value.value()).unwrap() //TODO don't unwrap
            });

        Ok(value)
    }

    fn get(cluster_id: ClusterId, storage: &Storage) -> PersistenceResult<Option<Self>> {
        let table = storage.db.open_table(CLUSTER_CONFIGURATION_TABLE).unwrap(); //TODO don't unwrap

        let key = cluster_id.0.as_bytes().as_slice();

        let value = table.get(key).unwrap() //TODO don't unwrap
            .map(|value| {
                serde_json::from_str::<ClusterConfiguration>(&value.value()).unwrap() //TODO don't unwrap
            });

        Ok(value)
    }

    fn list(storage: &Storage) -> PersistenceResult<HashMap<Self::Id, Self>> {
        let table = storage.db.open_table(CLUSTER_CONFIGURATION_TABLE).unwrap(); //TODO don't unwrap

        let value = table.iter().unwrap() //TODO don't unwrap
            .map(|value| {
                let (key, value) = value.unwrap(); //TODO don't unwrap
                let id = ClusterId::from(Uuid::from_slice(key.value()).unwrap()); //TODO don't unwrap

                let value = serde_json::from_str::<ClusterConfiguration>(&value.value()).unwrap(); //TODO don't unwrap

                (id, value)
            })
            .collect();

        Ok(value)
    }
}

const CLUSTER_CONFIGURATION_TABLE: TableDefinition<&[u8], String> = TableDefinition::new("cluster_configuration");

//TODO SerializableClusterConfiguration + Version-field
