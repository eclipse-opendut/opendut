use crate::viper::{ViperSourceId, ViperTestSuiteIdentifier};


/// Meta-information about a VIPER test suite,
/// including constraints for how it can be parametrized.
#[derive(Clone, Debug)]
pub struct ViperTestSuiteDescriptor {
    pub id: ViperTestSuiteIdentifier,
    pub source: ViperSourceId,
    /// Constraints for how the parameter values may look like.
    pub parameters: super::ViperParameterDescriptors,
}
