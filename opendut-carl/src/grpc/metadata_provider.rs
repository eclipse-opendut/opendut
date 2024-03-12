use tonic::{Request, Response, Status};
use tonic_web::CorsGrpcWeb;

use opendut_carl_api::proto::services::metadata_provider::{VersionRequest, VersionResponse};
use opendut_carl_api::proto::services::metadata_provider::metadata_provider_server::{MetadataProvider, MetadataProviderServer};
use opendut_types::proto::util::VersionInfo;

#[derive(Debug, Default)]
pub struct MetadataProviderFacade {}

impl MetadataProviderFacade {

    pub fn new() -> Self {
        Self { }
    }

    pub fn into_grpc_service(self) -> CorsGrpcWeb<MetadataProviderServer<Self>> {
        tonic_web::enable(MetadataProviderServer::new(self))
    }
}

#[tonic::async_trait]
impl MetadataProvider for MetadataProviderFacade {
    async fn version(
        &self,
        request: Request<VersionRequest>,
    ) -> Result<Response<VersionResponse>, Status> {

        log::trace!("Received request: {:?}", request);

        let reply = VersionResponse {
            version_info: Some(VersionInfo {
                name: String::from(crate::app_info::CRATE_VERSION),
            })
        };

        Ok(Response::new(reply))
    }
}
