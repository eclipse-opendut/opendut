use std::collections::HashMap;

use crate::viper::{ViperSourceId, ViperTestParameterKey, ViperTestSuiteIdentifier};


pub struct ViperTestSuiteDescriptor {
    pub id: ViperTestSuiteIdentifier,
    pub source: ViperSourceId,
    pub parameters: HashMap<ViperTestParameterKey, ViperTestParameterValueKind>,
}

pub enum ViperTestParameterValueKind {
    Boolean,
    Number,
    Text,
}
