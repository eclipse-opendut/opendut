use opendut_carl_api::carl::viper::{DeleteViperTestRunDescriptorError, GetViperTestRunDescriptorError};
use opendut_carl_api::carl::{CarlClient, ClientError};
use opendut_model::viper::ViperTestId;

/// Delete a VIPER test run descriptor
#[derive(clap::Parser)]
pub struct DeleteViperTestRunDescriptorCli {
    /// ID of the test
    #[arg()]
    id: ViperTestId,
    /// Report an error status code when the resource to delete does not exist
    #[arg(long)]
    error_when_missing: bool,
}

impl DeleteViperTestRunDescriptorCli {
    pub async fn execute(self, carl: &mut CarlClient) -> crate::Result<()> {
        let id = self.id;

        match carl.viper.get_viper_test_run_descriptor(id).await {
            Ok(_) => {}
            Err(ClientError::UsageError(GetViperTestRunDescriptorError::TestNotFound { .. }))
                if !self.error_when_missing => {
                    eprintln!("No VIPER test run descriptor found with ID <{id}>.");
                    return Ok(());
                }
            Err(error) => {
                return Err(format!("Failed to get VIPER test run descriptor with ID <{id}>.\n  {error}"));
            }
        }

        let deployments = carl.viper.list_viper_run_deployments().await
            .map_err(|error| format!("Failed to list VIPER run deployments.\n  {error}"))?;

        if let Some(deployment) = deployments.iter().find(|deployment| deployment.test_id == id) {
            return Err(format!("VIPER test <{id}> cannot be deleted while run deployment <{}> uses it.", deployment.run_id));
        }

        match carl.viper.delete_viper_test_run_descriptor(id).await {
            Ok(test_id) => {
                eprintln!("Deleted VIPER test run descriptor <{test_id}> successfully.");
                Ok(())
            }
            Err(error) => match error {
                ClientError::UsageError(DeleteViperTestRunDescriptorError::TestNotFound { .. }) => {
                    eprintln!("No VIPER test run descriptor found with ID <{id}>.");
                    if self.error_when_missing {
                        Err(error)
                    } else {
                        Ok(())
                    }
                }
                other => Err(other),
            }
        }.map_err(|error| format!("Failed to delete VIPER test run descriptor with ID <{id}>.\n  {error}"))
    }
}
