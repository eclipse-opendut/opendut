use opendut_model::viper::{ViperSourceDescriptor, ViperSourceId, ViperTestSuiteDescriptor};
use opendut_viper_rt::common::TestSuiteIdentifier;
use opendut_viper_rt::compile::IdentifierFilter;
use opendut_viper_rt::events::emitter;
use opendut_viper_rt::source::Source;
use opendut_viper_rt::ViperRuntime;
use crate::resource::{api::resources::Resources, persistence::error::PersistenceError, storage::ResourcesStorageApi};


impl Resources<'_> {
    #[tracing::instrument(skip_all, level="trace")]
    pub async fn get_viper_test_suite_descriptor(&self, source_id: ViperSourceId) -> Result<Option<ViperTestSuiteDescriptor>, GetViperTestSuiteDescriptorError> {

        let source = self.get::<ViperSourceDescriptor>(source_id)
            .map_err(|cause| GetViperTestSuiteDescriptorError::Persistence { source_id, cause })?;

        match source {
            Some(source) => discover_suite(source).await,
            None => Ok(None),
        }
    }
}


async fn discover_suite(source: ViperSourceDescriptor) -> Result<Option<ViperTestSuiteDescriptor>, GetViperTestSuiteDescriptorError>  {
    let ViperSourceDescriptor { id: source_id, name, url } = source;

    let name = TestSuiteIdentifier::try_from(name.value())
        .expect("Conversion of source name to TestSuiteIdentifier failed."); //FIXME ViperSourceDescriptor should use TestSuiteIdentifier directly, making this conversion obsolete


    let source = Source::from_url(name, url);


    let compilation = futures::executor::block_on(async { //Rustpython's types cannot be sent between threads (see e.g. the `NonNull` type), so not even wrapping in a `Mutex` allows them to be used in the async context underneath the gRPC interface, and we instead opt for explicit blocking of the thread until completion, which means the types will not get sent between threads.
        let viper_runtime = ViperRuntime::default();

        viper_runtime.compile(&source, &mut emitter::drain(), &IdentifierFilter::default()).await
    })
    .expect("Compilation failed"); //FIXME introduce error case


    Ok(Some(ViperTestSuiteDescriptor {
        id: compilation.identifier().to_owned(),
        source: source_id,
        parameters: compilation.parameters().to_owned(),
    }))
}


#[derive(thiserror::Error, Debug)]
pub enum GetViperTestSuiteDescriptorError {
    #[error("Error when accessing persistence while getting VIPER test suite descriptor for source <{source_id}>")]
    Persistence {
        source_id: ViperSourceId,
        #[source] cause: PersistenceError,
    },
}
