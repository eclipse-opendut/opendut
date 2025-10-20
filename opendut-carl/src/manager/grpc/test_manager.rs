use tonic::{Request, Response, Status};
use tracing::trace;
use opendut_carl_api::proto::services::test_manager::{store_test_suite_source_descriptor_response, StoreTestSuiteSourceDescriptorRequest, StoreTestSuiteSourceDescriptorResponse, StoreTestSuiteSourceDescriptorSuccess};
use opendut_carl_api::proto::services::test_manager::test_manager_server::{TestManager as TestManagerService, TestManagerServer};
use opendut_model::test_suite::TestSuiteSourceDescriptor;
use crate::manager::grpc::error::LogApiErr;
use crate::manager::grpc::extract;
use crate::resource::manager::ResourceManagerRef;
use crate::resource::persistence::error::PersistenceError;

pub struct TestManagerFacade {
    pub resource_manager: ResourceManagerRef,
}

impl TestManagerFacade {
    pub fn into_grpc_service(self) -> super::web::CorsGrpcWeb<TestManagerServer<Self>> {
        super::web::enable(TestManagerServer::new(self))
    }
}

#[tonic::async_trait]
impl TestManagerService for TestManagerFacade {

    #[tracing::instrument(skip_all, level="trace")]
    async fn store_test_suite_source_descriptor(&self, request: Request<StoreTestSuiteSourceDescriptorRequest>) -> Result<Response<StoreTestSuiteSourceDescriptorResponse>, Status> {

        let request = request.into_inner();
        let source: TestSuiteSourceDescriptor = extract!(request.source)?;

        trace!("Received request to store test suite source descriptor: {source:?}");


        let result =
            self.resource_manager.insert(source.id, source.clone()).await
                .log_api_err()
                .map_err(|_: PersistenceError| opendut_carl_api::carl::test_suite::StoreTestSuiteSourceDescriptorError::Internal {
                    source_id: source.id,
                    source_name: source.name,
                    cause: String::from("Error when accessing persistence while storing test suite source descriptor"),
                });

        let reply = match result {
            Ok(()) => store_test_suite_source_descriptor_response::Reply::Success(
                StoreTestSuiteSourceDescriptorSuccess {
                    source_id: Some(source.id.into()),
                }
            ),
            Err(error) => store_test_suite_source_descriptor_response::Reply::Failure(error.into()),
        };

        Ok(Response::new(StoreTestSuiteSourceDescriptorResponse {
            reply: Some(reply),
        }))
    }
}
