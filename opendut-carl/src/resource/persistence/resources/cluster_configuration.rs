use super::Persistable;
use crate::resource::api::id::ResourceId;
use crate::resource::persistence;
use crate::resource::persistence::error::PersistenceResult;
use crate::resource::persistence::{Db, Memory, TableDefinition};
use opendut_types::cluster::{ClusterConfiguration, ClusterId};
use prost::Message;
use std::collections::HashMap;

const CLUSTER_CONFIGURATION_TABLE: TableDefinition = TableDefinition::new("cluster_configuration");


impl Persistable for ClusterConfiguration {
    fn insert(self, cluster_id: ClusterId, _: &mut Memory, db: &Db) -> PersistenceResult<()> {
        let key = persistence::Key::from(ResourceId::<Self>::into_id(cluster_id));

        let value = opendut_types::proto::cluster::ClusterConfiguration::from(self)
            .encode_to_vec();

        let mut table = db.read_write_table(CLUSTER_CONFIGURATION_TABLE)?;
        table.insert(key, value)?;

        Ok(())
    }

    fn remove(cluster_id: ClusterId, _: &mut Memory, db: &Db) -> PersistenceResult<Option<Self>> {
        let key = persistence::Key::from(ResourceId::<Self>::into_id(cluster_id));

        let mut table = db.read_write_table(CLUSTER_CONFIGURATION_TABLE)?;

        let value = table.remove(key)?
            .map(|value| try_from_bytes(value.value()))
            .transpose()?;

        Ok(value)
    }

    fn get(cluster_id: ClusterId, _: &Memory, db: &Db) -> PersistenceResult<Option<Self>> {
        let key = persistence::Key::from(ResourceId::<Self>::into_id(cluster_id));

        if let Some(table) = db.read_table(CLUSTER_CONFIGURATION_TABLE)? {
            let value = table.get(&key)?
                .map(|value| try_from_bytes(value.value()))
                .transpose()?;
            Ok(value)
        } else {
            Ok(None)
        }
    }

    fn list(_: &Memory, db: &Db) -> PersistenceResult<HashMap<Self::Id, Self>> {
        if let Some(table) = db.read_table(CLUSTER_CONFIGURATION_TABLE)? {
            table.iter()?
                .map(|value| {
                    let (key, value) = value?;
                    let id = ResourceId::<Self>::from_id(key.value().id);
                    let value = try_from_bytes(value.value())?;

                    Ok((id, value))
                })
                .collect::<PersistenceResult<HashMap<_, _>>>()
        } else {
            Ok(HashMap::default())
        }
    }
}

fn try_from_bytes(bytes: Vec<u8>) -> PersistenceResult<ClusterConfiguration> {
    let value = opendut_types::proto::cluster::ClusterConfiguration::decode(bytes.as_slice())?;
    let value = ClusterConfiguration::try_from(value)?;
    Ok(value)
}
