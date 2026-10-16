use indoc::indoc;

use opendut_carl_api::carl::CarlClient;
use opendut_model::viper::{ViperRunDeployment, ViperRunId};

use crate::DescribeOutputFormat;
use super::SerializableViperRunDeployment;

/// Describe a VIPER run deployment
#[derive(clap::Parser)]
pub struct DescribeViperRunDeploymentCli {
    /// ID of the run
    #[arg()]
    id: ViperRunId,
}

impl DescribeViperRunDeploymentCli {
    pub async fn execute(self, carl: &mut CarlClient, output: DescribeOutputFormat) -> crate::Result<()> {
        let run_id = self.id;

        let deployment = carl.viper.get_viper_run_deployment(run_id).await
            .map_err(|error| format!("Failed to retrieve VIPER run deployment <{run_id}>.\n  {error}"))?;

        render_viper_run_deployment(deployment, output)
    }
}

pub fn render_viper_run_deployment(deployment: ViperRunDeployment, output: DescribeOutputFormat) -> crate::Result<()> {
    let deployment = SerializableViperRunDeployment::from(deployment);

    let text = match output {
        DescribeOutputFormat::Text => {
            format!(indoc!("
                VIPER Run Deployment:
                  RunID: {}
                  TestID: {}"), deployment.run_id, deployment.test_id)
        }
        DescribeOutputFormat::Json => {
            serde_json::to_string(&deployment)
                .map_err(|error| format!("Could not serialize VIPER run deployment.\n  {error}"))?
        }
        DescribeOutputFormat::PrettyJson => {
            serde_json::to_string_pretty(&deployment)
                .map_err(|error| format!("Could not serialize VIPER run deployment.\n  {error}"))?
        }
    };

    println!("{text}");
    Ok(())
}
