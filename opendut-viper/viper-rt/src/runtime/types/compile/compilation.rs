use crate::common::TestSuiteIdentifier;
use crate::runtime::types::compile::metadata::Metadata;
use crate::runtime::types::compile::parameters::ParameterDescriptors;
use crate::runtime::types::compile::suite::TestSuite;
use std::fmt::Debug;

/// A `Compilation` is the outcome of the [`ViperRuntime's`][ViperRuntime] [`compile`] function.
///
/// The `Compilation` is used as a container and consists of the following parts:
/// - [`Metadata`]
/// - [`ParameterDescriptors`]
/// - [`TestSuite`]
///
/// The different parts are accessible through a shared reference by their respective getter functions.
/// Alternatively, a `Compilation` can be split to take ownership of the parts:
/// ```
/// # use indoc::indoc;
/// # use opendut_viper_rt::events::emitter;
/// # use opendut_viper_rt::run::{Report, Outcome, ParameterBindings};
/// # use opendut_viper_rt::source::Source;
/// # use opendut_viper_rt::ViperRuntime;
/// # use opendut_viper_rt::compile::IdentifierFilter;
/// #
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// #
/// #   let viper = ViperRuntime::default();
/// #
/// #   let source = Source::embedded(
/// #       indoc!(r#"
/// #           # VIPER_VERSION = 1.0
/// #           from viper import *
/// #
/// #           class MyTestCase(unittest.TestCase):
/// #               def test_something():
/// #                   self.assertEquals(7+3, 10)
/// #       "#)
/// #   );
/// #
/// let compilation = viper.compile(&source, &mut emitter::drain(), &IdentifierFilter::default()).await?;
///
/// let metadata = compilation.metadata();
/// let parameters = compilation.parameters();
/// let suite = compilation.suite();
///
/// let (metadata, parameters, suite) = compilation.split();
/// #
/// #   Ok(())
/// # }
/// ```
///
/// [ViperRuntime]: crate::runtime::ViperRuntime
/// [`compile`]: crate::runtime::ViperRuntime::compile
///
pub struct Compilation {
    metadata: Metadata,
    parameters: ParameterDescriptors,
    suite: TestSuite,
}

impl Compilation {
    #[allow(dead_code)]
    pub(crate) fn new(metadata: Metadata, parameters: ParameterDescriptors, suite: TestSuite) -> Self {
        Self { metadata, parameters, suite }
    }

    pub fn identifier(&self) -> &TestSuiteIdentifier {
        self.suite.identifier()
    }

    pub fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    pub fn parameters(&self) -> &ParameterDescriptors {
        &self.parameters
    }

    pub fn suite(&self) -> &TestSuite {
        &self.suite
    }

    pub fn split(self) -> (Metadata, ParameterDescriptors, TestSuite) {
        (self.metadata, self.parameters, self.suite)
    }

    pub fn into_suite(self) -> TestSuite {
        self.suite
    }
}

impl Debug for Compilation {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Compilation")
            .finish()
    }
}
