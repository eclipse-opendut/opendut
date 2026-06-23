use anyhow::Context;
use tracing::info;
use opendut_model::viper::{ViperSourceDescriptor, ViperTestId, ViperTestRunDescriptor};
use opendut_viper_rt::source::Source;
use opendut_viper_rt::{ViperOptions, ViperRuntime};
use opendut_viper_rt::compile::{CompilationError, SourceCode};
use opendut_viper_rt::source::loaders::HttpSourceLoader;
use crate::resource::manager::{Resources, error::PersistenceError, ResourcesStorageApi};


impl Resources<'_> {
    pub async fn fetch_source_code(
        &self,
        test_id: ViperTestId,
        fetch_source_code: impl AsyncFnOnce(ViperRuntime, &Source) -> Result<SourceCode, FetchError>,
    ) -> Result<(), FetchError> {
        let viper_test = self.get::<ViperTestRunDescriptor>(test_id)?
            .context(format!("VIPER test descriptor <{test_id}> not found."))?;

        let viper_source_id = viper_test.source;

        let viper_source = self.get::<ViperSourceDescriptor>(viper_source_id)?
            .context(format!("VIPER source descriptor <{viper_source_id}> not found."))?;

        let test_suite_identifier = viper_source.name;
        let url = viper_source.url;

        let source = Source::from_url(test_suite_identifier, url);

        let viper_runtime = ViperRuntime::new(ViperOptions {
            source_loaders: vec![Box::new(HttpSourceLoader)],
            ..Default::default()
        }).unwrap();

        let source_code = fetch_source_code(viper_runtime, &source).await?;

        info!("Fetched source code from source <{}>: \n{:#}", source_code.identifier, source_code.code);
        Ok(())
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

        resource_manager.resources_mut(async |resources| {
            let test_id = viper_test_fixture.id;
            resources.fetch_source_code(test_id, mock_fetch_source_code).await
        }).await??;

        Ok(())
    }
}

#[derive(thiserror::Error, Debug)]
pub enum FetchError {
    #[error(transparent)]
    ResourceManager(#[from] anyhow::Error),

    #[error(transparent)]
    Persistence(#[from] PersistenceError),

    #[error("failed to fetch source code")]
    FetchSourceCode(#[from] Box<CompilationError>),
}
