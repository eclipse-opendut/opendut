use cli_table::{print_stdout, Table, WithTitle};
use opendut_carl_api::carl::CarlClient;
use opendut_types::cluster::{ClusterId};
use crate::ListOutputFormat;

#[derive(Table)]
struct ClusterTable {
    #[table(title = "ClusterID")]
    id: ClusterId,
}

pub async fn execute(carl: &mut CarlClient, output: ListOutputFormat) -> crate::Result<()> {

    let clusters = carl.cluster.list_cluster_deployments().await
        .map_err(|error| format!("Error while listing cluster deployments: {}", error))?;

    match output {
        ListOutputFormat::Table => {
            let cluster_table = clusters.into_iter()
                .map(|cluster_deployment| {
                    ClusterTable {
                        id: cluster_deployment.id,
                    }
                })
                .collect::<Vec<_>>();
            print_stdout(cluster_table.with_title())
                .expect("List of clusters should be printable as table.");
        }
        ListOutputFormat::Json => {
            let json = serde_json::to_string(&clusters).unwrap();
            println!("{}", json);
        }
        ListOutputFormat::PrettyJson => {
            let json = serde_json::to_string_pretty(&clusters).unwrap();
            println!("{}", json);
        }
    }

    Ok(())
}
