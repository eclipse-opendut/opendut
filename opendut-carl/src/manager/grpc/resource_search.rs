use tonic::{Request, Response, Status};
use tracing::{error, trace};

use opendut_carl_api::carl::resource_search::SearchError;
use opendut_carl_api::proto::services::resource_search::{resource_match, search_response, ResourceMatch, SearchRequest, SearchResponse, SearchSuccess};
use opendut_carl_api::proto::services::resource_search::resource_search_server::{ResourceSearch, ResourceSearchServer};
use opendut_model::cluster::ClusterDescriptor;
use opendut_model::peer::PeerDescriptor;

use crate::resource::manager::ResourceManagerRef;

pub struct ResourceSearchFacade {
    resource_manager: ResourceManagerRef,
}

impl ResourceSearchFacade {

    pub fn new(resource_manager: ResourceManagerRef) -> Self {
        Self { resource_manager }
    }

    pub fn into_grpc_service(self) -> super::web::CorsGrpcWeb<ResourceSearchServer<Self>> {
        super::web::enable(ResourceSearchServer::new(self))
    }

}

#[tonic::async_trait]
impl ResourceSearch for ResourceSearchFacade {

    #[tracing::instrument(skip_all, level="trace")]
    async fn search(&self, request: Request<SearchRequest>) -> Result<Response<SearchResponse>, Status> {

        let query = request.into_inner().query.to_lowercase();

        trace!("Received request to search resources matching '{query}'.");

        let mut matches: Vec<ResourceMatch> = Vec::new();

        let peers_result = self.resource_manager.list::<PeerDescriptor>().await
            .inspect_err(|error| error!("Error while listing peer descriptors from gRPC API: {error}"))
            .map_err(|cause| SearchError::Internal { cause: cause.to_string() });

        let reply = match peers_result {
            Err(error) => search_response::Reply::Failure(error.into()),
            Ok(peers) => {
                for peer in peers.into_values() {
                    for device in &peer.topology.devices {
                        if device.name.value().to_lowercase().contains(&query) {
                            matches.push(ResourceMatch {
                                resource: Some(resource_match::Resource::Device(device.clone().into())),
                            });
                        }
                    }
                    if peer.name.value().to_lowercase().contains(&query) {
                        matches.push(ResourceMatch {
                            resource: Some(resource_match::Resource::Peer(peer.into())),
                        });
                    }
                }

                let clusters_result = self.resource_manager.list::<ClusterDescriptor>().await
                    .inspect_err(|error| error!("Error while listing cluster descriptors from gRPC API: {error}"))
                    .map_err(|cause| SearchError::Internal { cause: cause.to_string() });

                match clusters_result {
                    Err(error) => search_response::Reply::Failure(error.into()),
                    Ok(clusters) => {
                        for cluster in clusters.into_values() {
                            if cluster.name.value().to_lowercase().contains(&query) {
                                matches.push(ResourceMatch {
                                    resource: Some(resource_match::Resource::Cluster(cluster.into())),
                                });
                            }
                        }
                        search_response::Reply::Success(SearchSuccess { matches })
                    }
                }
            }
        };

        Ok(Response::new(SearchResponse { reply: Some(reply) }))
    }
}
