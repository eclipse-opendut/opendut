use cli_table::{print_stdout, Table, WithTitle};

use opendut_carl_api::carl::CarlClient;
use opendut_model::cluster::{ClusterId, ClusterName};

use crate::ListOutputFormat;

/// List all cluster descriptors
#[derive(clap::Parser)]
pub struct ListClusterDescriptorsCli;

#[derive(Table)]
struct ClusterTable {
    #[table(title = "Name")]
    name: ClusterName,
    #[table(title = "ClusterID")]
    id: ClusterId,
}

impl ListClusterDescriptorsCli {
    pub async fn execute(self, carl: &mut CarlClient, output: ListOutputFormat) -> crate::Result<()> {
        let clusters = carl.cluster.list_cluster_descriptors().await
            .map_err(|error| format!("Could not list any cluster descriptors.\n  {error}"))?;

        match output {
            ListOutputFormat::Table => {
                let cluster_table = clusters.into_iter()
                    .map(|cluster| {
                        ClusterTable {
                            name: cluster.name,
                            id: cluster.id,
                        }
                    })
                    .collect::<Vec<_>>();
                print_stdout(cluster_table.with_title())
                    .expect("List of cluster descriptors should be printable as table.");
            }
            ListOutputFormat::Json => {
                let json = serde_json::to_string(&clusters).unwrap();
                println!("{json}");
            }
            ListOutputFormat::PrettyJson => {
                let json = serde_json::to_string_pretty(&clusters).unwrap();
                println!("{json}");
            }
        }
        Ok(())
    }
}
