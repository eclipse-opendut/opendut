use tracing::trace;
use opendut_model::viper::{ViperSourceDescriptor, ViperSourceId, ViperTestId, ViperTestRunDescriptor};
use opendut_viper_rt::source::{Source, SourceLocation};
use opendut_viper_rt::{ViperOptions, ViperRuntime};
use opendut_viper_rt::common::TestSuiteIdentifier;
use opendut_viper_rt::compile::{CompilationError, SourceCode};
use opendut_viper_rt::source::loaders::HttpSourceLoader;
use crate::resource::manager::{ResourceManagerRef, Resources, ResourcesStorageApi};
use crate::resource::manager::error::PersistenceError;

pub async fn fetch_source_code(
    resource_manager: ResourceManagerRef,
    test_id: ViperTestId,
    fetch_source_code: impl AsyncFnOnce(ViperRuntime, &Source) -> Result<SourceCode, FetchError>,
) -> Result<SourceCode, FetchError> {

    let viper_source = resource_manager.resources(async |resources| {
        resources.get_viper_source_descriptor(test_id)
    }).await
        .map_err(|error| FetchError::Persistence { test_id, source: error })??;

    let test_suite_identifier = viper_source.name;
    let url = viper_source.url;

    let source = Source::from_url(test_suite_identifier, url);

    let viper_runtime = ViperRuntime::new(ViperOptions {
        source_loaders: vec![Box::new(HttpSourceLoader)],
        ..Default::default()
    }).unwrap(); // Todo: don't unwrap()

    let source_code = fetch_source_code(viper_runtime, &source).await?;

    trace!("Fetched source code from source <{}>: \n{:#}", source_code.identifier, source_code.code);
    Ok(source_code)
}

impl Resources<'_> {
    fn get_viper_source_descriptor(
        &self,
        test_id: ViperTestId,
    ) -> Result<ViperSourceDescriptor, FetchError> {
        let viper_test = self.get::<ViperTestRunDescriptor>(test_id)
            .map_err(|error| FetchError::Persistence { test_id, source: error })?
            .ok_or_else(|| FetchError::ViperTestRunDescriptorNotFound { test_id })?;

        let viper_source_id = viper_test.source;

        let viper_source = self.get::<ViperSourceDescriptor>(viper_source_id)
            .map_err(|error| FetchError::Persistence { test_id, source: error })?
            .ok_or_else(|| FetchError::ViperSourceDescriptorNotFound { test_id, source_id: viper_source_id })?;

        Ok(viper_source)
    }
}


#[cfg(test)]
mod test {
    use crate::manager::testing::{SourceCodeFixture, ViperTestFixture};
    use crate::resource::manager::ResourceManager;
    use super::*;

    #[tokio::test]
    async fn test_fetch_source_code() -> anyhow::Result<()> {
        let (resource_manager, _resource_manager_cancel) = ResourceManager::new_in_memory();
        let viper_test_fixture = ViperTestFixture::create(resource_manager.clone()).await?;

        let mock_fetch_source_code = async move |_viper_runtime: ViperRuntime, _source: &Source| {
            let source_code = SourceCodeFixture::new().source_code;
            Ok(source_code)
        };

        let test_id = viper_test_fixture.id;
        fetch_source_code(resource_manager, test_id, mock_fetch_source_code).await?;

        Ok(())
    }
}

#[derive(thiserror::Error, Debug)]
pub enum FetchError {

    #[error("Source code for test <{test_id}> could not be fetched, because a test with that ID does not exist!")]
    ViperTestRunDescriptorNotFound {
        test_id: ViperTestId,
    },

    #[error("Source code for test <{test_id}> could not be fetched, because a source with the ID <{source_id}> does not exist!")]
    ViperSourceDescriptorNotFound {
        test_id: ViperTestId,
        source_id: ViperSourceId,
    },

    #[error("Source code for test <{test_id}> could not be fetched, because of an error when accessing persistence!")]
    Persistence {
        test_id: ViperTestId,
        source: PersistenceError,
    },

    #[error("Compilation failed while getting the source code for test suite <{test_suite_identifier}> with the location ({location:?})!")]
    Compilation {
        test_suite_identifier: TestSuiteIdentifier,
        location: SourceLocation,
        source: Box<CompilationError>,
    },
}
