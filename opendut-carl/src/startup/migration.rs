#![expect(deprecated)]

use crate::resource::storage::ResourcesStorageApi;
use std::ops::Not;
use tracing::{debug, info};
use crate::resource::api::id::ResourceId;
use crate::resource::api::Resource;
use crate::resource::persistence::persistable::{Persistable, StorageKind};
use opendut_model::cluster::{ClusterDescriptor, ClusterId};
use opendut_util::conversion;
use opendut_util::proto::ConversionResult;
use opendut_model::resources::Id;
use crate::resource::manager::ResourceManagerRef;
use crate::resource::persistence::error::PersistenceResult;


pub async fn run(resource_manager: ResourceManagerRef) -> anyhow::Result<()> {
    debug!("Running migrations...");
    resave_cluster_configurations_as_cluster_descriptors(resource_manager).await?;
    debug!("Migrations complete.");
    Ok(())
}

async fn resave_cluster_configurations_as_cluster_descriptors(resource_manager: ResourceManagerRef) -> anyhow::Result<()> {
    resource_manager.resources_mut(async |resources| {
        let cluster_configurations = resources.list::<ClusterConfiguration>()?;

        if cluster_configurations.is_empty().not() {
            info!("Running cluster configuration to descriptor migration.");

            for (cluster_id, cluster_configuration) in cluster_configurations {
                resources.insert::<ClusterDescriptor>(cluster_id, cluster_configuration.inner)?;

                resources.remove::<ClusterConfiguration>(cluster_id)?;
            }
        }

        PersistenceResult::Ok(())
    }).await??;
    Ok(())
}

#[deprecated]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ClusterConfiguration {
    inner: ClusterDescriptor,
}

impl ResourceId<ClusterConfiguration> for ClusterId {
    fn into_id(self) -> Id {
        Id::from(self.0)
    }

    fn from_id(id: Id) -> Self {
        ClusterId::from(id.value())
    }
}

impl Resource for ClusterConfiguration {
    type Id = ClusterId;
}

impl Persistable for ClusterConfiguration {
    type Proto = opendut_model::proto::cluster::ClusterDescriptor;
    const TABLE: &'static str = "cluster_configuration";
    const STORAGE: StorageKind = StorageKind::Persistent;
}


conversion! {
    type Model = ClusterConfiguration;
    type Proto = opendut_model::proto::cluster::ClusterDescriptor;

    fn from(value: Model) -> Proto {
        value.inner.into()
    }

    fn try_from(value: Proto) -> ConversionResult<Model> {
        Ok(Model {
            inner: ClusterDescriptor::try_from(value)?
        })
    }
}
