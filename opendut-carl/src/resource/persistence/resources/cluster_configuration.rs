use super::Persistable;
use crate::resource::persistence::error::PersistenceResult;
use crate::resource::persistence::{Memory, Db};
use opendut_types::cluster::{ClusterConfiguration, ClusterId};
use std::collections::HashMap;
use redb::TableDefinition;
use crate::resource::api::id::ResourceId;
use crate::resource::persistence;

impl Persistable for ClusterConfiguration {
    fn insert(self, cluster_id: ClusterId, _: &mut Memory, db: &Db) -> PersistenceResult<()> {
        let key = persistence::Key::from(ResourceId::<Self>::into_id(cluster_id));

        let value = self;
        let value = serde_json::to_string(&value).unwrap(); //TODO don't unwrap

        let mut table = db.read_write_table(CLUSTER_CONFIGURATION_TABLE)?;
        table.insert(key, value).unwrap(); //TODO don't unwrap

        Ok(())
    }

    fn remove(cluster_id: ClusterId, _: &mut Memory, db: &Db) -> PersistenceResult<Option<Self>> {
        let key = persistence::Key::from(ResourceId::<Self>::into_id(cluster_id));

        let mut table = db.read_write_table(CLUSTER_CONFIGURATION_TABLE)?;

        let value = table.remove(key).unwrap() //TODO don't unwrap
            .map(|value| {
                serde_json::from_str::<ClusterConfiguration>(&value.value()).unwrap() //TODO don't unwrap
            });

        Ok(value)
    }

    fn get(cluster_id: ClusterId, _: &Memory, db: &Db) -> PersistenceResult<Option<Self>> {
        let key = persistence::Key::from(ResourceId::<Self>::into_id(cluster_id));

        let value = db.read_table(CLUSTER_CONFIGURATION_TABLE)?
            .and_then(|table| {
                table.get(&key).unwrap() //TODO don't unwrap
                    .map(|value| {
                        serde_json::from_str::<ClusterConfiguration>(&value.value()).unwrap() //TODO don't unwrap
                    })
            });

        Ok(value)
    }

    fn list(_: &Memory, db: &Db) -> PersistenceResult<HashMap<Self::Id, Self>> {
        let value = db.read_table(CLUSTER_CONFIGURATION_TABLE)?
            .map(|table| {
                table.iter().unwrap() //TODO don't unwrap
                    .map(|value| {
                        let (key, value) = value.unwrap(); //TODO don't unwrap
                        let id = ResourceId::<Self>::from_id(key.value().id);

                        let value = serde_json::from_str::<ClusterConfiguration>(&value.value()).unwrap(); //TODO don't unwrap

                        (id, value)
                    })
                    .collect::<HashMap<_, _>>()
            })
            .unwrap_or_default();

        Ok(value)
    }
}

const CLUSTER_CONFIGURATION_TABLE: persistence::TableDefinition = TableDefinition::new("cluster_configuration");

//TODO SerializableClusterConfiguration + Version-field
