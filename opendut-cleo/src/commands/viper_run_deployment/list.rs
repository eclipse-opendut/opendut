use cli_table::{print_stdout, Table, WithTitle};

use opendut_carl_api::carl::CarlClient;
use opendut_model::viper::{ViperRunId, ViperTestId};

use crate::OutputFormat;
use super::SerializableViperRunDeployment;

/// List all VIPER run deployments
#[derive(clap::Parser)]
pub struct ListViperRunDeploymentsCli;

#[derive(Table)]
struct ViperRunDeploymentTable {
    #[table(title = "RunID")]
    run_id: ViperRunId,
    #[table(title = "TestID")]
    test_id: ViperTestId,
}

impl From<SerializableViperRunDeployment> for ViperRunDeploymentTable {
    fn from(deployment: SerializableViperRunDeployment) -> Self {
        let SerializableViperRunDeployment { run_id, test_id } = deployment;

        Self { run_id, test_id }
    }
}

impl ListViperRunDeploymentsCli {
    pub async fn execute(self, carl: &mut CarlClient, output: OutputFormat) -> crate::Result<()> {
        let deployments = carl.viper.list_viper_run_deployments().await
            .map_err(|error| format!("Could not list VIPER run deployments.\n  {error}"))?;

        let serializable_deployments = deployments.into_iter()
            .map(SerializableViperRunDeployment::from)
            .collect::<Vec<_>>();

        match output {
            OutputFormat::Table => {
                let deployment_table = serializable_deployments.into_iter()
                    .map(ViperRunDeploymentTable::from)
                    .collect::<Vec<_>>();

                print_stdout(deployment_table.with_title())
                    .map_err(|error| format!("Could not print VIPER run deployments.\n  {error}"))?;
            }
            OutputFormat::Json => {
                let json = serde_json::to_string(&serializable_deployments)
                    .map_err(|error| format!("Could not serialize VIPER run deployments.\n  {error}"))?;
                println!("{json}");
            }
            OutputFormat::PrettyJson => {
                let json = serde_json::to_string_pretty(&serializable_deployments)
                    .map_err(|error| format!("Could not serialize VIPER run deployments.\n  {error}"))?;
                println!("{json}");
            }
        }

        Ok(())
    }
}
