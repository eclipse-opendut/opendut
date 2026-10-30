use opendut_carl_api::carl::CarlClient;
use opendut_model::viper::{ViperRunDeployment, ViperRunId, ViperTestId};
use crate::output::OutputFormat;
use super::SerializableViperRunDeployment;

/// Create a VIPER run deployment to request execution of a test
#[derive(clap::Parser)]
pub struct CreateViperRunDeploymentCli {
    /// ID of the test to deploy
    #[arg(long)]
    test_id: ViperTestId,
    /// ID of the run; generates a random UUID when omitted
    #[arg(short, long)]
    id: Option<ViperRunId>,
}

impl CreateViperRunDeploymentCli {
    pub async fn execute(self, carl: &mut CarlClient, output: OutputFormat) -> crate::Result<()> {
        let test_id = self.test_id;

        carl.viper.get_viper_test_run_descriptor(test_id).await
            .map_err(|error| format!("Failed to retrieve VIPER test run descriptor <{test_id}>.\n  {error}"))?;

        let deployment = ViperRunDeployment {
            run_id: self.id.unwrap_or_else(ViperRunId::random),
            test_id,
        };

        create_viper_run_deployment(deployment, carl, &output).await
    }
}

pub async fn create_viper_run_deployment(
    deployment: ViperRunDeployment,
    carl: &mut CarlClient,
    output: &OutputFormat,
) -> crate::Result<()> {
    carl.viper.store_viper_run_deployment(deployment.clone()).await
        .map_err(|error| format!("Could not store VIPER run deployment <{}>.\n  {error}", deployment.run_id))?;

    let deployment = SerializableViperRunDeployment::from(deployment);

    match output {
        OutputFormat::Table => {
            println!("Stored VIPER run deployment <{}> for test <{}>.", deployment.run_id, deployment.test_id);
        }
        OutputFormat::Json => {
            let json = serde_json::to_string(&deployment)
                .map_err(|error| format!("Could not serialize VIPER run deployment.\n  {error}"))?;
            println!("{json}");
        }
        OutputFormat::PrettyJson => {
            let json = serde_json::to_string_pretty(&deployment)
                .map_err(|error| format!("Could not serialize VIPER run deployment.\n  {error}"))?;
            println!("{json}");
        }
    }

    Ok(())
}
