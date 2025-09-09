use tonic::{Request, Response, Status};
use tracing::trace;

use opendut_carl_api::proto::services::metadata_provider::{VersionRequest, VersionResponse};
use opendut_carl_api::proto::services::metadata_provider::metadata_provider_server::{MetadataProvider, MetadataProviderServer};
use opendut_model::proto::util::VersionInfo;

#[derive(Debug, Default)]
pub struct MetadataProviderFacade {}

impl MetadataProviderFacade {

    pub fn new() -> Self {
        Self { }
    }

    pub fn into_grpc_service(self) -> super::web::CorsGrpcWeb<MetadataProviderServer<Self>> {
        super::web::enable(MetadataProviderServer::new(self))
    }
}

#[tonic::async_trait]
impl MetadataProvider for MetadataProviderFacade {

    #[tracing::instrument(skip_all, level="trace")]
    async fn version(&self, _: Request<VersionRequest>) -> Result<Response<VersionResponse>, Status> {

        trace!("Received request to get version information.");

        let reply = VersionResponse {
            version_info: Some(VersionInfo {
                name: String::from(crate::app_info::PKG_VERSION),
                revision: String::from(crate::app_info::COMMIT_HASH),
                revision_date: String::from(crate::app_info::COMMIT_DATE),
                build_date: String::from(crate::app_info::BUILD_TIME),
            })
        };

        Ok(Response::new(reply))
    }
}
