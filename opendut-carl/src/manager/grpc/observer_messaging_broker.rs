use std::collections::HashSet;
use std::ops::Not;
use std::pin::Pin;
use futures::Stream;
use tokio_stream::StreamExt;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};
use tonic_web::CorsGrpcWeb;
use tracing::trace;
use opendut_carl_api::proto::services::observer_messaging_broker::observer_messaging_broker_server::ObserverMessagingBrokerServer;
use opendut_carl_api::proto::services::observer_messaging_broker::observer_messaging_broker_server::{ObserverMessagingBroker as ObserverMessagingBrokerService};
use opendut_carl_api::proto::services::observer_messaging_broker::{WaitForPeersOnlineRequest, WaitForPeersOnlineResponse};
use opendut_types::peer::{PeerDescriptor, PeerId};
use opendut_types::proto::ConversionError;
use crate::manager::observer_messaging_broker::ObserverMessagingBrokerRef;
use crate::resource::manager::ResourceManagerRef;

pub struct ObserverMessagingBrokerFacade {
    resource_manager: ResourceManagerRef,
    observer_messaging_broker: ObserverMessagingBrokerRef,
}

impl ObserverMessagingBrokerFacade {

    pub fn new(resource_manager: ResourceManagerRef, observer_messaging_broker: ObserverMessagingBrokerRef) -> Self {
        Self {
            resource_manager,
            observer_messaging_broker,
        }
    }

    pub fn into_grpc_service(self) -> CorsGrpcWeb<ObserverMessagingBrokerServer<Self>> {
        tonic_web::enable(ObserverMessagingBrokerServer::new(self))
    }

}

impl ObserverMessagingBrokerFacade {
    async fn check_peer_ids_exist_and_nonempty(&self, peer_ids: &HashSet<PeerId>) -> Result<(), Status> {
        if peer_ids.is_empty() {
            return Err(Status::invalid_argument("No peer IDs provided"))
        }

        let all_peer_ids = self.resource_manager.list::<PeerDescriptor>().await
            .map_err(|error| Status::internal(error.to_string()))?
            .keys().cloned().collect::<HashSet<_>>();
        let unknown_peers = peer_ids.difference(&all_peer_ids).cloned().collect::<Vec<_>>();
        if unknown_peers.is_empty().not() {
            return Err(Status::invalid_argument(format!("Unknown peer IDs provided: <{unknown_peers:?}>.")))
        }
        Ok(())
    }
}

#[tonic::async_trait]
impl ObserverMessagingBrokerService for ObserverMessagingBrokerFacade {
    type WaitForPeersOnlineStream = Pin<Box<dyn Stream<Item = Result<WaitForPeersOnlineResponse, Status>> + Send>>;

    async fn wait_for_peers_online(&self, request: Request<WaitForPeersOnlineRequest>) -> Result<Response<Self::WaitForPeersOnlineStream>, Status> {
        let request = request.into_inner();
        let peer_ids = request.peer_ids.into_iter()
            .map(PeerId::try_from)
            .collect::<Result<HashSet<PeerId>, ConversionError>>()
            .map_err(|error| Status::internal(error.to_string()))?;
        self.check_peer_ids_exist_and_nonempty(&peer_ids).await?;

        trace!("Received request to wait for following peers to be online <{:?}>.", peer_ids);
        let rx_outbound = self.observer_messaging_broker.wait_for_peers_online(peer_ids).await
            .map_err(|cause| Status::internal(cause.to_string()))?;
        let outbound_stream = ReceiverStream::new(rx_outbound)
            .map(Ok);

        Ok(Response::new(
            Box::pin(outbound_stream) as Self::WaitForPeersOnlineStream
        ))
    }

}
