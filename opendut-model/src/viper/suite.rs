use crate::viper::{ViperSourceId, ViperTestSuiteIdentifier};


pub struct ViperTestSuiteDescriptor {
    pub id: ViperTestSuiteIdentifier,
    pub source: ViperSourceId,
    pub parameters: super::ViperParameterDescriptors,
}
