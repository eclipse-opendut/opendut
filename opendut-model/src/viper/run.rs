use std::collections::HashMap;
use opendut_viper_rt::common::TestSuiteIdentifier;
use crate::create_id_type;
use crate::viper::ViperSourceId;


#[derive(Clone, Debug)]
pub struct ViperRunDescriptor {
    pub id: ViperRunId,
    pub source: ViperSourceId,
    pub suite: TestSuiteIdentifier,
    pub parameters: HashMap<ViperRunParameterKey, ViperRunParameterValue>,
}


create_id_type!(ViperRunId);


#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ViperRunParameterKey { pub inner: String }

#[derive(Clone, Debug)]
pub enum ViperRunParameterValue {
    Boolean(bool),
    Number(i64),
    Text(String),
}
