use opendut_model::test::suite::{TestSuiteSourceId, TestSuiteSourceName};

#[derive(thiserror::Error, Debug)]
pub enum StoreTestSuiteSourceDescriptorError {
    #[error("Test suite source descriptor '{source_name}' <{source_id}> could not be created, due to internal errors:\n  {cause}")]
    Internal {
        source_id: TestSuiteSourceId,
        source_name: TestSuiteSourceName,
        cause: String
    }
}
