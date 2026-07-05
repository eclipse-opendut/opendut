use opendut_model::{format::format_id_with_optional_name, viper::{ViperSourceDescriptor, ViperSourceId, ViperTestRunDescriptor, ViperTestId}};
use tracing::debug;
use opendut_model::viper::ViperTestSuiteIdentifier;
use crate::resource::{types::resources::Resources, persistence::error::PersistenceError, manager::ResourcesStorageApi};


impl Resources<'_> {
    #[tracing::instrument(skip_all, level="trace")]
    pub async fn delete_viper_source_descriptor(&mut self, source_id: ViperSourceId) -> Result<ViperSourceDescriptor, DeleteViperSourceDescriptorError> {

        debug!("Fetching list of VIPER tests that might be using VIPER source <{source_id}>.");
        let tests = self.list::<ViperTestRunDescriptor>()
            .map_err(|cause| DeleteViperSourceDescriptorError::Persistence { source_id, source_name: None, cause })?;

        match tests.values().find(|test| test.source == source_id) {
            None => {
                debug!("No tests are using VIPER source <{source_id}>. Continuing with deletion of source.");
                let result = self.remove::<ViperSourceDescriptor>(source_id)
                    .map_err(|cause: PersistenceError|
                        DeleteViperSourceDescriptorError::Persistence { source_id, source_name: None, cause }
                    )?
                    .ok_or_else(|| DeleteViperSourceDescriptorError::SourceNotFound { source_id })?;

                Ok(result)
            }
            Some(test) => {
                debug!("Test <{}> and potentially others are using VIPER source <{source_id}>. Aborting deletion of source.", test.id);
                Err(DeleteViperSourceDescriptorError::TestExists { source_id, test_id: test.id })
            }
        }
    }
}


#[derive(thiserror::Error, Debug)]
pub enum DeleteViperSourceDescriptorError {
    #[error("Source <{source_id}> could not be deleted, because a source with that ID does not exist!")]
    SourceNotFound {
        source_id: ViperSourceId,
    },
    #[error("VIPER Source <{source_id}> could not be deleted, because a VIPER test <{test_id}> using this source still exists!")]
    TestExists {
        source_id: ViperSourceId,
        test_id: ViperTestId,
    },
    #[error("Error when accessing persistence while deleting VIPER source descriptor for source {source}", source=format_id_with_optional_name(source_id, source_name))]
    Persistence {
        source_id: ViperSourceId,
        source_name: Option<ViperTestSuiteIdentifier>,
        #[source] cause: PersistenceError,
    },
}



#[cfg(test)]
mod tests {
    use crate::manager::testing::{ViperSourceFixture, ViperTestFixture};
    use crate::resource::manager::ResourceManager;
    use super::*;

    #[tokio::test]
    async fn delete_source_descriptor_when_no_test_is_using_it() -> anyhow::Result<()> {
        // Arrange
        let (resource_manager, _cancel) = ResourceManager::new_in_memory();
        let source = ViperSourceFixture::create(resource_manager.clone()).await?;

        // Act
        let result = resource_manager.resources_mut(async |resources|
            resources.delete_viper_source_descriptor(source.id).await
        ).await??;

        // Assert
        assert_eq!(result, source.descriptor);

        Ok(())
    }

    #[tokio::test]
    async fn block_deletion_of_source_descriptor_if_a_test_with_this_source_is_still_deployed() -> anyhow::Result<()> {
        // Arrange
        let (resource_manager, _cancel) = ResourceManager::new_in_memory();
        let test = ViperTestFixture::create(resource_manager.clone()).await?;

        // Act
        let result = resource_manager.resources_mut(async |resources|
            resources.delete_viper_source_descriptor(test.descriptor.source).await
        ).await?;

        // Assert
        let Err(DeleteViperSourceDescriptorError::TestExists { source_id, test_id }) = result
        else { panic!("Result is not an error of DeleteViperSourceDescriptorError.") };

        assert_eq!(source_id, test.descriptor.source);
        assert_eq!(test_id, test.id);

        Ok(())
    }
}
