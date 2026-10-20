use opendut_carl_api::carl::viper::DeleteViperSourceDescriptorError;
use opendut_carl_api::carl::{CarlClient, ClientError};
use opendut_model::viper::ViperSourceId;

/// Delete a VIPER source descriptor
#[derive(clap::Parser)]
pub struct DeleteViperSourceDescriptorCli {
    /// ID of the source
    #[arg()]
    id: ViperSourceId,
    /// Report an error status code when the resource to delete does not exist
    #[arg(long)]
    error_when_missing: bool,
}

impl DeleteViperSourceDescriptorCli {
    pub async fn execute(self, carl: &mut CarlClient) -> crate::Result<()> {
        let id = self.id;

        match carl.viper.delete_viper_source_descriptor(id).await {
            Ok(source_id) => {
                eprintln!("Deleted VIPER source descriptor <{source_id}> successfully.");
                Ok(())
            }
            Err(error) => match error {
                ClientError::UsageError(DeleteViperSourceDescriptorError::SourceNotFound { .. }) => {
                    eprintln!("No VIPER source descriptor found with ID <{id}>.");
                    if self.error_when_missing {
                        Err(error)
                    } else {
                        Ok(())
                    }
                }
                other => Err(other),
            }
        }.map_err(|error| format!("Failed to delete VIPER source descriptor with ID <{id}>.\n  {error}"))
    }
}
