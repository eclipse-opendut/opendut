use anyhow::bail;
use crate::resource::storage::{PersistenceOptions, ResourcesStorageApi};
use tracing::{debug, trace};
use opendut_types::cluster::{ClusterConfiguration, ClusterDeployment};
use opendut_types::peer::PeerDescriptor;
use crate::resource::manager::ResourceManagerRef;
use crate::resource::persistence::error::PersistenceResult;
use crate::resource::persistence::query::Filter;


pub(crate) async fn load_data_from_postgres_into_key_value_store(resource_manager: ResourceManagerRef, persistence_options: &PersistenceOptions) -> anyhow::Result<()> {
    match persistence_options {
        PersistenceOptions::Enabled { database_connect_info } => {
            let database_result = crate::resource::persistence::database::connect(database_connect_info).await;

            match database_result {
                Ok(mut database) => {
                    let postgres_peer_descriptors = crate::resource::persistence::query::peer_descriptor::list(Filter::Not, &mut database)?;
                    let postgres_cluster_configurations = crate::resource::persistence::query::cluster_configuration::list(Filter::Not, &mut database)?;
                    let postgres_cluster_deployments = crate::resource::persistence::query::cluster_deployment::list(Filter::Not, &mut database)?;

                    let (redb_peer_descriptors, redb_cluster_configurations, redb_cluster_deployments) =
                        resource_manager.resources(async |resources| {
                            let peer_descriptors = resources.list::<PeerDescriptor>()?;
                            let cluster_configurations = resources.list::<ClusterConfiguration>()?;
                            let cluster_deployments = resources.list::<ClusterDeployment>()?;

                            PersistenceResult::Ok((peer_descriptors, cluster_configurations, cluster_deployments))
                        }).await??;


                    if let Some(id) = postgres_peer_descriptors.keys().find(|id| redb_peer_descriptors.contains_key(id)) {
                        bail!("Conflicting peer descriptor <{id}> while attempting migration from PostgreSQL. Aborting without changes.");
                    }
                    if let Some(id) = postgres_cluster_configurations.keys().find(|id| redb_cluster_configurations.contains_key(id)) {
                        bail!("Conflicting cluster configuration <{id}> while attempting migration from PostgreSQL. Aborting without changes.");
                    }
                    if let Some(id) = postgres_cluster_deployments.keys().find(|id| redb_cluster_deployments.contains_key(id)) {
                        bail!("Conflicting cluster deployment <{id}> while attempting migration from PostgreSQL. Aborting without changes.");
                    }

                    resource_manager.resources_mut(async |resources| {
                        for (id, value) in postgres_peer_descriptors {
                            resources.insert(id, value)?;
                        }
                        for (id, value) in postgres_cluster_configurations {
                            resources.insert(id, value)?;
                        }
                        for (id, value) in postgres_cluster_deployments {
                            resources.insert(id, value)?;
                        }

                        PersistenceResult::Ok(())
                    }).await??;
                }
                Err(cause) => {
                    debug!("No PostgreSQL database reachable at {}. Not performing migration.\n  {cause}", database_connect_info.url);
                }
            };

        }
        PersistenceOptions::Disabled => {
            trace!("Persistence is disabled. Not performing migration from PostgreSQL.");
        }
    }

    Ok(())
}
