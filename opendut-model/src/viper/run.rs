use std::collections::HashMap;
use opendut_viper_rt::common::TestSuiteIdentifier;
use crate::create_id_type;
use crate::viper::TestSuiteSourceId;


#[derive(Clone, Debug)]
pub struct TestSuiteRunDescriptor {
    pub id: TestSuiteRunId,
    pub source: TestSuiteSourceId,
    pub suite: TestSuiteIdentifier,
    pub parameters: HashMap<TestSuiteRunParameterKey, TestSuiteRunParameterValue>,
}


create_id_type!(TestSuiteRunId);


#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TestSuiteRunParameterKey { pub inner: String }

#[derive(Clone, Debug)]
pub enum TestSuiteRunParameterValue {
    Boolean(bool),
    Number(i64),
    Text(String),
}
