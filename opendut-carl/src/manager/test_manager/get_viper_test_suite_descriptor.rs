use opendut_model::viper::{ViperSourceDescriptor, ViperSourceId, ViperTestSuiteDescriptor};

use crate::resource::{api::resources::Resources, persistence::error::PersistenceError, storage::ResourcesStorageApi};


impl Resources<'_> {
    #[tracing::instrument(skip_all, level="trace")]
    pub async fn get_viper_test_suite_descriptor(&self, source_id: ViperSourceId) -> Result<Option<ViperTestSuiteDescriptor>, GetViperTestSuiteDescriptorError> {

        let source = self.get::<ViperSourceDescriptor>(source_id)
            .map_err(|cause| GetViperTestSuiteDescriptorError::Persistence { source_id, cause })?;

        let suite = source.map(discover_suite);

        Ok(suite)
    }
}


fn discover_suite(_source: ViperSourceDescriptor) -> ViperTestSuiteDescriptor {
    todo!("call into VIPER to download source + discover suite parameters");
}


#[derive(thiserror::Error, Debug)]
pub enum GetViperTestSuiteDescriptorError {
    #[error("Error when accessing persistence while getting VIPER test suite descriptor for source <{source_id}>")]
    Persistence {
        source_id: ViperSourceId,
        #[source] cause: PersistenceError,
    },
}
