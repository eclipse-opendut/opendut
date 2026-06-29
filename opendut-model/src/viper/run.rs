use crate::create_id_type;
use crate::viper::ViperTestId;
use serde::{Serialize, Serializer};
use opendut_viper_rt::compile::SourceCode;


#[derive(Clone, Debug)]
pub struct ViperRunDeployment {
    pub run_id: ViperRunId,
    pub test_id: ViperTestId,
}

create_id_type!(ViperRunId);



#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TestRunSourceCode {
    pub inner: SourceCode,
}

// Limited implementation for debugging purposes
impl Serialize for TestRunSourceCode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        #[derive(Serialize)]
        struct Serializable {
            suite: String,
        }

        let SourceCode { identifier, code: _code, version: _version } = &self.inner;
        let serializable = Serializable { suite: identifier.to_string() };
        serializer.serialize_newtype_struct("TestRunSourceCode", &serializable)
    }
}
