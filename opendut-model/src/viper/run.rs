use std::collections::HashMap;
use viper_rt::common::TestSuiteIdentifier;
use crate::create_id_type;
use crate::viper::TestSuiteSourceId;


pub struct TestSuiteRunDescriptor {
    pub id: TestSuiteRunId,
    pub source: TestSuiteSourceId,
    pub suite: TestSuiteIdentifier,
    pub parameters: HashMap<TestSuiteRunParameterKey, TestSuiteRunParameterValue>,
}


create_id_type!(TestSuiteRunId);


#[derive(PartialEq, Eq, Hash)]
pub struct TestSuiteRunParameterKey { pub inner: String }

pub enum TestSuiteRunParameterValue {
    Boolean(bool),
    Number(i64),
    Text(String),
}
