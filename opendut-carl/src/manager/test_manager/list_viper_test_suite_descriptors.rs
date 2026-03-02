use opendut_model::viper::{ViperSourceDescriptor, ViperTestSuiteDescriptor};

use crate::resource::{api::resources::Resources, persistence::error::PersistenceError, storage::ResourcesStorageApi};


impl Resources<'_> {
    #[tracing::instrument(skip_all, level="trace")]
    pub async fn list_viper_test_suite_descriptors(&self) -> Result<Vec<ViperTestSuiteDescriptor>, ListViperTestSuiteDescriptorsError> {

        let sources = self.list::<ViperSourceDescriptor>()
            .map_err(|cause| ListViperTestSuiteDescriptorsError::Persistence { cause })?;

        let suites = sources.into_values()
            .flat_map(discover_suites)
            .collect::<Vec<_>>();

        Ok(suites)
    }
}


///TODO We should cache the results somehow and ideally pre-calculate them when a source is stored.
///     Otherwise, this request could easily time out, if VIPER has to download+compile all the sources before it can respond.
fn discover_suites(_source: ViperSourceDescriptor) -> Vec<ViperTestSuiteDescriptor> {
    todo!("call into VIPER to download source + discover suites");
}


#[derive(thiserror::Error, Debug)]
pub enum ListViperTestSuiteDescriptorsError {
    #[error("Error when accessing persistence while listing VIPER test suite descriptors")]
    Persistence {
        #[source] cause: PersistenceError,
    },
}
