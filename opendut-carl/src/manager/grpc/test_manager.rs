use tonic::{Request, Response, Status};
use tracing::{error, trace};
use opendut_carl_api::proto::services::test_manager::{delete_test_suite_source_descriptor_response, get_test_suite_source_descriptor_response, list_test_suite_source_descriptors_response, store_test_suite_source_descriptor_response, DeleteTestSuiteSourceDescriptorRequest, DeleteTestSuiteSourceDescriptorResponse, DeleteTestSuiteSourceDescriptorSuccess, GetTestSuiteSourceDescriptorRequest, GetTestSuiteSourceDescriptorResponse, GetTestSuiteSourceDescriptorSuccess, ListTestSuiteSourceDescriptorsRequest, ListTestSuiteSourceDescriptorsResponse, ListTestSuiteSourceDescriptorsSuccess, StoreTestSuiteSourceDescriptorRequest, StoreTestSuiteSourceDescriptorResponse, StoreTestSuiteSourceDescriptorSuccess};
use opendut_carl_api::proto::services::test_manager::test_manager_server::{TestManager as TestManagerService, TestManagerServer};
use opendut_model::viper::{TestSuiteSourceDescriptor, TestSuiteSourceId};
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
                .map_err(|_: PersistenceError| opendut_carl_api::carl::viper::StoreTestSuiteSourceDescriptorError::Internal {
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

    #[tracing::instrument(skip_all, level="trace")]
    async fn delete_test_suite_source_descriptor(&self, request: Request<DeleteTestSuiteSourceDescriptorRequest>) -> Result<Response<DeleteTestSuiteSourceDescriptorResponse>, Status> {

        let request = request.into_inner();
        let source_id: TestSuiteSourceId = extract!(request.source_id)?;

        trace!("Received request to delete test suite source descriptor for source <{source_id}>.");

        let result =
            self.resource_manager.remove::<TestSuiteSourceDescriptor>(source_id).await
                .log_api_err()
                .map_err(|_: PersistenceError| opendut_carl_api::carl::viper::DeleteTestSuiteSourceDescriptorError::Internal {
                    source_id,
                    source_name: None,
                    cause: String::from("Error when accessing persistence while storing test suite source descriptor"),
                });

        let response = match result {
            Ok(_) => delete_test_suite_source_descriptor_response::Reply::Success(
                DeleteTestSuiteSourceDescriptorSuccess {
                    source_id: Some(source_id.into())
                }
            ),
            Err(error) => delete_test_suite_source_descriptor_response::Reply::Failure(error.into()),
        };

        Ok(Response::new(DeleteTestSuiteSourceDescriptorResponse {
            reply: Some(response),
        }))
    }

    #[tracing::instrument(skip_all, level="trace")]
    async fn get_test_suite_source_descriptor(&self, request: Request<GetTestSuiteSourceDescriptorRequest>) -> Result<Response<GetTestSuiteSourceDescriptorResponse>, Status> {

        let request = request.into_inner();
        let source_id: TestSuiteSourceId = extract!(request.source_id)?;

        trace!("Received request to get test suite source descriptor for source <{source_id}>.");

        let result =
            self.resource_manager.get::<TestSuiteSourceDescriptor>(source_id).await
                .inspect_err(|error| error!("Error while getting test suite source descriptor from gRPC API: {error}"))
                .map_err(|_: PersistenceError| opendut_carl_api::carl::viper::GetTestSuiteSourceDescriptorError::Internal {
                    source_id,
                    cause: String::from("Error when accessing persistence while getting test suite source descriptor"),
                });

        let response = match result {
            Ok(descriptor) => match descriptor {
                Some(descriptor) => get_test_suite_source_descriptor_response::Reply::Success(
                    GetTestSuiteSourceDescriptorSuccess {
                        descriptor: Some(descriptor.into())
                    }
                ),
                None => get_test_suite_source_descriptor_response::Reply::Failure(
                    opendut_carl_api::carl::viper::GetTestSuiteSourceDescriptorError::SourceNotFound { source_id }.into()
                ),
            }
            Err(error) => get_test_suite_source_descriptor_response::Reply::Failure(error.into()),
        };

        Ok(Response::new(GetTestSuiteSourceDescriptorResponse {
            reply: Some(response)
        }))
    }

    #[tracing::instrument(skip_all, level="trace")]
    async fn list_test_suite_source_descriptors(&self, _: Request<ListTestSuiteSourceDescriptorsRequest>) -> Result<Response<ListTestSuiteSourceDescriptorsResponse>, Status> {

        trace!("Received request to list test suite source descriptors.");

        let result = self.resource_manager.list::<TestSuiteSourceDescriptor>().await
            .inspect_err(|error| error!("Error while listing test suite source descriptors from gRPC API: {error}"))
            .map_err(|_: PersistenceError| opendut_carl_api::carl::viper::ListTestSuiteSourceDescriptorsError::Internal {
                cause: String::from("Error when accessing persistence while listing test suite source descriptors"),
            });

        let response = match result {
            Ok(sources) => {
                let sources = sources.into_values()
                    .map(From::from)
                    .collect::<Vec<_>>();

                list_test_suite_source_descriptors_response::Reply::Success(
                    ListTestSuiteSourceDescriptorsSuccess { sources }
                )
            }
            Err(error) => list_test_suite_source_descriptors_response::Reply::Failure(error.into())
        };

        Ok(Response::new(ListTestSuiteSourceDescriptorsResponse {
            reply: Some(response)
        }))
    }
}
