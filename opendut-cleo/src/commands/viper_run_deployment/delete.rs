use opendut_carl_api::carl::viper::{DeleteViperRunDeploymentError, GetViperRunDeploymentError};
use opendut_carl_api::carl::{CarlClient, ClientError};
use opendut_model::viper::ViperRunId;

/// Delete a VIPER run deployment
#[derive(clap::Parser)]
pub struct DeleteViperRunDeploymentCli {
    /// ID of the run
    #[arg()]
    id: ViperRunId,
    /// Report an error status code when the resource to delete does not exist
    #[arg(long)]
    error_when_missing: bool,
}

impl DeleteViperRunDeploymentCli {
    pub async fn execute(self, carl: &mut CarlClient) -> crate::Result<()> {
        let id = self.id;

        match carl.viper.get_viper_run_deployment(id).await {
            Ok(_) => {}
            Err(ClientError::UsageError(GetViperRunDeploymentError::RunDeploymentNotFound { .. }))
                if !self.error_when_missing => {
                    eprintln!("No VIPER run deployment found with ID <{id}>.");
                    return Ok(());
                }
            Err(error) => {
                return Err(format!("Failed to get VIPER run deployment with ID <{id}>.\n  {error}"));
            }
        }

        match carl.viper.delete_viper_run_deployment(id).await {
            Ok(run_id) => {
                eprintln!("Deleted VIPER run deployment <{run_id}> successfully.");
                Ok(())
            }
            Err(error) => match error {
                ClientError::UsageError(DeleteViperRunDeploymentError::RunDeploymentNotFound { .. }) => {
                    eprintln!("No VIPER run deployment found with ID <{id}>.");
                    if self.error_when_missing {
                        Err(error)
                    } else {
                        Ok(())
                    }
                }
                other => Err(other),
            }
        }.map_err(|error| format!("Failed to delete VIPER run deployment with ID <{id}>.\n  {error}"))
    }
}
