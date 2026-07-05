use opendut_model::{viper::{ViperTestRunDescriptor, ViperTestId}};
use tracing::debug;
use opendut_model::viper::{ViperRunDeployment, ViperRunId};
use crate::resource::{types::resources::Resources, persistence::error::PersistenceError, storage::ResourcesStorageApi};


impl Resources<'_> {
    #[tracing::instrument(skip_all, level="trace")]
    pub async fn delete_viper_test_descriptor(&mut self, test_id: ViperTestId) -> Result<ViperTestRunDescriptor, DeleteViperTestDescriptorError> {

        debug!("Fetching list of VIPER run deployments that might be using VIPER test <{test_id}>.");
        let runs = self.list::<ViperRunDeployment>()
            .map_err(|cause| DeleteViperTestDescriptorError::Persistence { test_id, cause })?;

        match runs.values().find(|run| run.test_id == test_id) {
            None => {
                debug!("No run deployments are using VIPER test <{test_id}>. Continuing with deletion of test descriptor.");
                let result = self
                    .remove::<ViperTestRunDescriptor>(test_id)
                    .map_err(|cause: PersistenceError| {
                        DeleteViperTestDescriptorError::Persistence { test_id, cause }
                    })?
                    .ok_or_else(|| DeleteViperTestDescriptorError::TestNotFound { test_id })?;

                Ok(result)
            }
            Some(run) => {
                debug!("Run deployment <{}> and potentially others are using VIPER test <{test_id}>. Aborting deletion of test descriptor.",run.run_id);
                Err(DeleteViperTestDescriptorError::ViperRunDeploymentExists {
                    test_id,
                    run_id: run.run_id,
                })
            }
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum DeleteViperTestDescriptorError {
    #[error("Test <{test_id}> could not be deleted, because a test with that ID does not exist!")]
    TestNotFound {
        test_id: ViperTestId,
    },

    #[error("Test <{test_id}> could not be deleted, because a viper run deployment <{run_id}> using this test still exists!")]
    ViperRunDeploymentExists {
        test_id: ViperTestId,
        run_id: ViperRunId,
    },

    #[error("Error when accessing persistence while deleting VIPER test descriptor for test <{test_id}>.")]
    Persistence {
        test_id: ViperTestId,
        #[source]
        cause: PersistenceError,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manager::testing::{ViperRunDeploymentFixture, ViperTestFixture};
    use crate::resource::manager::ResourceManager;

    #[tokio::test]
    async fn delete_test_descriptor_when_no_run_deployment_is_using_it() -> anyhow::Result<()> {
        // Arrange
        let (resource_manager, _cancel) = ResourceManager::new_in_memory();
        let test = ViperTestFixture::create(resource_manager.clone()).await?;

        // Act
        let result = resource_manager.resources_mut(async |resources|
            resources.delete_viper_test_descriptor(test.id).await
        ).await??;

        // Assert
        assert_eq!(result, test.descriptor);

        Ok(())
    }

    #[tokio::test]
    async fn block_deletion_of_test_descriptor_if_a_run_deployment_is_still_using_it() -> anyhow::Result<()> {
        // Arrange
        let (resource_manager, _cancel) = ResourceManager::new_in_memory();
        let run_deployment = ViperRunDeploymentFixture::create(resource_manager.clone()).await?;

        // Act
        let result = resource_manager.resources_mut(async |resources|
            resources.delete_viper_test_descriptor(run_deployment.deployment.test_id).await
        ).await?;

        // Assert
        let Err(DeleteViperTestDescriptorError::ViperRunDeploymentExists { test_id, run_id }) = result
        else { panic!("Result is not an error of DeleteViperTestDescriptorError.") };

        assert_eq!(test_id, run_deployment.deployment.test_id);
        assert_eq!(run_id, run_deployment.id);

        Ok(())
    }
}
