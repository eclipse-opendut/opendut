use tracing::debug;
use opendut_model::viper::{ViperSourceDescriptor, ViperSourceId, ViperTestSuiteIdentifier, ViperTestSuiteParameters};
use opendut_viper_rt::compile::{IdentifierFilter};
use opendut_viper_rt::events::emitter;
use opendut_viper_rt::source::Source;
use opendut_viper_rt::{ViperOptions, ViperRuntime};
use opendut_viper_rt::source::loaders::HttpSourceLoader;
use crate::resource::manager::{Resources, error::PersistenceError, ResourcesStorageApi};


impl Resources<'_> {
    #[tracing::instrument(skip_all, level="trace")]
    pub async fn get_viper_test_suite_parameters(&self, source_id: ViperSourceId) -> Result<Option<ViperTestSuiteParameters>, GetViperTestSuiteParametersError> {

        let source = self.get::<ViperSourceDescriptor>(source_id)
            .map_err(|source| GetViperTestSuiteParametersError::Persistence { source_id, source })?;

        match source {
            Some(source) => discover_suite(source).await,
            None => Ok(None),
        }
    }
}


async fn discover_suite(source: ViperSourceDescriptor) -> Result<Option<ViperTestSuiteParameters>, GetViperTestSuiteParametersError>  {
    let ViperSourceDescriptor { id: source_id, name: test_suite_identifier, url, .. } = source;

    let source = Source::from_url(Clone::clone(&test_suite_identifier), url);

    debug!("Calling VIPER to compile source into test suite.");

    let handle = tokio::task::spawn_blocking(move || {
        tokio::runtime::Handle::current().block_on(async move {
            // The ..Default::default() is needed here because ViperOptions has additional fields (e.g. container_runtime) under other feature flags. Removing it breaks the --all-features build.
            #[allow(clippy::needless_update)]
            let viper_runtime = ViperRuntime::new(ViperOptions {
                source_loaders: vec![Box::new(HttpSourceLoader)],
                ..Default::default()
            }).map_err(|_| GetViperTestSuiteParametersError::ViperRuntime { source_id, source_name: Clone::clone(&test_suite_identifier) })?;

            let compilation = viper_runtime.compile(&source, &mut emitter::drain(), &IdentifierFilter::default()).await
                .map_err(|_| GetViperTestSuiteParametersError::Compilation { source_id, source_name: Clone::clone(&test_suite_identifier) })?;

            Ok((compilation.identifier().to_owned(), compilation.parameters().to_owned()))
        })
    });

    debug!("VIPER compilation completed.");

    let (identifier, parameters) = handle.await
        .map_err(|source| GetViperTestSuiteParametersError::TaskJoin { source_id, when: "compiling VIPER source", source })??;

    Ok(Some(ViperTestSuiteParameters {
        id: identifier,
        parameters,
    }))
}


#[derive(thiserror::Error, Debug)]
pub enum GetViperTestSuiteParametersError {
    #[error("Compilation failed while getting VIPER test suite descriptor for source {source_name} with ID <{source_id}>.")]
    Compilation {
        source_id: ViperSourceId,
        source_name: ViperTestSuiteIdentifier,
    },
    #[error("Async task failed when {when} while getting VIPER test suite descriptor for source <{source_id}>.")]
    TaskJoin {
        source_id: ViperSourceId,
        when: &'static str,
        source: tokio::task::JoinError,
    },
    #[error("Error while initializing VIPER runtime for VIPER test source {source_name} with ID <{source_id}>.")]
    ViperRuntime {
        source_id: ViperSourceId,
        source_name: ViperTestSuiteIdentifier,
    },
    #[error("Error when accessing persistence while getting VIPER test suite descriptor for source <{source_id}>")]
    Persistence {
        source_id: ViperSourceId,
        source: PersistenceError,
    },
}
