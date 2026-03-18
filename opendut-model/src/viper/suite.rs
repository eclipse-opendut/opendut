use crate::viper::{ViperSourceId, ViperTestSuiteIdentifier};


#[derive(Clone, Debug)]
pub struct ViperTestSuiteDescriptor {
    pub id: ViperTestSuiteIdentifier,
    pub source: ViperSourceId,
    pub parameters: super::ViperParameterDescriptors,
}
