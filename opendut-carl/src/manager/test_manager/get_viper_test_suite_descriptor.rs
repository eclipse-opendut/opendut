use std::collections::HashMap;
use opendut_model::viper::{ViperSourceDescriptor, ViperSourceId, ViperTestParameterKey, ViperTestParameterValueKind, ViperTestSuiteDescriptor};
use opendut_viper_rt::common::TestSuiteIdentifier;
use opendut_viper_rt::compile::{IdentifierFilter, ParameterDescriptor};
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
    let viper_runtime = ViperRuntime::default();
    let ViperSourceDescriptor { id: source_id, name, url } = source;

    let name = TestSuiteIdentifier::try_from(name.value())
        .expect("Conversion of source name to TestSuiteIdentifier failed."); //FIXME ViperSourceDescriptor should use TestSuiteIdentifier directly, making this conversion obsolete


    let source = Source::from_url(name, url);

    let compilation = viper_runtime.compile(&source, &mut emitter::drain(), &IdentifierFilter::default()).await
        .expect("Compilation failed"); //FIXME introduce error case

    let parameters = compilation.parameters().iter()
        .map(|parameter_descriptor| {
            let key = ViperTestParameterKey { inner: parameter_descriptor.name().to_string() };

            let value_kind = match parameter_descriptor { //TODO use ParameterDescriptors in CARL-API
                ParameterDescriptor::BooleanParameter { .. } => ViperTestParameterValueKind::Boolean,
                ParameterDescriptor::NumberParameter { .. } => ViperTestParameterValueKind::Number,
                ParameterDescriptor::TextParameter { .. } => ViperTestParameterValueKind::Text,
            };

            (key, value_kind)
        })
        .collect::<HashMap<_, _>>();

    Ok(Some(ViperTestSuiteDescriptor {
        id: compilation.identifier().to_owned(),
        source: source_id,
        parameters,
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
