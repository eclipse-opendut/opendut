use anyhow::Context;
use tracing::info;
use opendut_model::viper::{ViperSourceDescriptor, ViperTestId, ViperTestRunDescriptor};
use opendut_viper_rt::source::Source;
use opendut_viper_rt::{ViperOptions, ViperRuntime};
use opendut_viper_rt::compile::{CompilationError, SourceCode};
use opendut_viper_rt::source::loaders::HttpSourceLoader;
use crate::resource::manager::{Resources, error::PersistenceError, ResourcesStorageApi};


impl Resources<'_> {
    pub async fn fetch_source_code(&self, test_id: ViperTestId) -> Result<(), FetchError> {
        let fetch_source_code = async move |viper_runtime: ViperRuntime, source: &Source| {
            let source_code = viper_runtime.fetch_source_code(source).await?;
            Ok(source_code)
        };

        self.fetch_source_code_impl(test_id, fetch_source_code).await?;

        Ok(())
    }

    async fn fetch_source_code_impl(
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
    use std::collections::HashMap;
    use url::Url;
    use opendut_model::cluster::ClusterId;
    use opendut_model::viper::{ViperSourceId, ViperSourceKind, ViperTestName, ViperTestSuiteIdentifier};
    use opendut_viper_rt::common::TestSuiteIdentifier;
    use opendut_viper_rt::compile::ApiVersion;
    use crate::resource::manager::{ResourceManager, ResourceManagerCancel, ResourceManagerRef};
    use super::*;

    #[tokio::test]
    async fn test_fetch_source_code() -> anyhow::Result<()> {
        let mut fixture = Fixture::create().await;
        let test_id = ViperTestId::random();
        let source_id = ViperSourceId::random();

        fixture.insert_test_data(test_id, source_id).await?;

        let mock_fetch_source_code = async move |_viper_runtime: ViperRuntime, _source: &Source| {
            let source_code = SourceCode {
                identifier: TestSuiteIdentifier::try_from("TestSuite")
                    .expect("Invalid TestSuiteIdentifier!"),
                code: String::from("print(Hello World!)"),
                version: ApiVersion::V1_0,
            };
            Ok(source_code)
        };

        fixture.resource_manager.resources_mut(async |resources| {
           resources.fetch_source_code_impl(test_id, mock_fetch_source_code).await
        }).await??;

        Ok(())
    }

    struct Fixture {
        resource_manager: ResourceManagerRef,
        _resource_manager_cancel: ResourceManagerCancel, // Carried along, so that it's dropped at the end of the test.
    }

    impl Fixture {
        async fn create() -> Fixture {
            let (resource_manager, _resource_manager_cancel) = ResourceManager::new_in_memory();

            Fixture {
                resource_manager,
                _resource_manager_cancel,
            }
        }

        async fn insert_test_data(&mut self, test_id: ViperTestId, source_id: ViperSourceId) -> anyhow::Result<()> {
            let test_run_descriptor = ViperTestRunDescriptor {
                id: test_id,
                name: ViperTestName::try_from("VIPER")?,
                source: source_id,
                cluster: ClusterId::random(),
                parameters: HashMap::new(),
            };
            self.resource_manager.insert::<ViperTestRunDescriptor>(test_id, test_run_descriptor)
                .await?;

            let viper_source_descriptor = ViperSourceDescriptor {
                id: source_id,
                name: ViperTestSuiteIdentifier::try_from("VIPER")?,
                url: Url::try_from("https://example.com/")?,
                kind: ViperSourceKind::Http,
            };
            self.resource_manager.insert::<ViperSourceDescriptor>(source_id, viper_source_descriptor)
                .await?;

            Ok(())
        }
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
