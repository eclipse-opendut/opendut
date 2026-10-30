use std::collections::HashSet;
use std::fmt::Display;
use cli_table::{print_stdout, Table, WithTitle};
use serde::Serialize;
use opendut_carl_api::carl::CarlClient;
use opendut_model::cluster::{ClusterDescriptor, ClusterId, ClusterName};

use crate::OutputFormat;

/// List all cluster descriptors
#[derive(clap::Parser)]
pub struct ListClusterDescriptorsCli;

#[derive(Serialize)]
struct SerializableCluster {
    #[serde(flatten)]
    descriptor: ClusterDescriptor,
    status: ClusterStatus,
}

#[derive(Table)]
struct ClusterTable {
    #[table(title = "Name")]
    name: ClusterName,
    #[table(title = "ClusterID")]
    id: ClusterId,
    #[table(title = "Status")]
    status: ClusterStatus,
}

#[derive(Debug, PartialEq, Serialize)]
enum ClusterStatus {
    Deployed,
    Undeployed,
}

impl Display for ClusterStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Self::Deployed => write!(f, "Deployed"),
            Self::Undeployed => write!(f, "Undeployed"),
        }
    }
}

impl ListClusterDescriptorsCli {
    pub async fn execute(self, carl: &mut CarlClient, output: OutputFormat) -> crate::Result<()> {
        let clusters = carl.cluster.list_cluster_descriptors().await
            .map_err(|error| format!("Could not list any cluster descriptors.\n  {error}"))?;

        let deployed_clusters = carl.cluster.list_cluster_deployments().await
            .map_err(|error| format!("Could not list cluster deployments.\n  {error}"))?
            .into_iter()
            .map(|deployment| deployment.id)
            .collect::<HashSet<_>>();

        let clusters = clusters.into_iter()
            .map(|cluster| {
                let status = if deployed_clusters.contains(&cluster.id) {
                    ClusterStatus::Deployed
                } else {
                    ClusterStatus::Undeployed
                };

                SerializableCluster { descriptor: cluster, status }
            })
            .collect::<Vec<_>>();

        match output {
            OutputFormat::Table => {
                let cluster_table = clusters.into_iter()
                    .map(|cluster| {
                        ClusterTable {
                            name: cluster.descriptor.name,
                            id: cluster.descriptor.id,
                            status: cluster.status,
                        }
                    })
                    .collect::<Vec<_>>();
                print_stdout(cluster_table.with_title())
                    .expect("List of cluster descriptors should be printable as table.");
            }
            OutputFormat::Json => {
                let json = serde_json::to_string(&clusters).unwrap();
                println!("{json}");
            }
            OutputFormat::PrettyJson => {
                let json = serde_json::to_string_pretty(&clusters).unwrap();
                println!("{json}");
            }
        }
        Ok(())
    }
}
