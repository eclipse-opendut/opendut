use crate::cluster::ClusterId;
use crate::viper::TestSuiteRunId;


#[derive(Clone, Debug)]
pub struct TestSuiteRunDeployment {
    pub id: TestSuiteRunId,
    pub cluster: ClusterId,
}
