use std::pin::Pin;

use futures::StreamExt;
use tokio_stream::Stream;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status, Streaming};
use tonic_web::CorsGrpcWeb;
use uuid::Uuid;

use opendut_carl_api::proto::services::peer_messaging_broker::{Downstream, ListPeersRequest, ListPeersResponse, Upstream};
use opendut_carl_api::proto::services::peer_messaging_broker::peer_messaging_broker_server::PeerMessagingBrokerServer;
use opendut_types::peer::PeerId;

use crate::peer::broker::broker::PeerMessagingBrokerRef;

pub struct PeerMessagingBrokerFacade {
    peer_messaging_broker: PeerMessagingBrokerRef,
}

impl PeerMessagingBrokerFacade {
    pub fn new(peer_messaging_broker: PeerMessagingBrokerRef) -> Self {
        Self { peer_messaging_broker }
    }
    pub fn into_grpc_service(self) -> CorsGrpcWeb<PeerMessagingBrokerServer<Self>> {
        tonic_web::enable(PeerMessagingBrokerServer::new(self))
    }
}

#[tonic::async_trait]
impl opendut_carl_api::proto::services::peer_messaging_broker::peer_messaging_broker_server::PeerMessagingBroker for PeerMessagingBrokerFacade {

    async fn list_peers(&self, request: Request<ListPeersRequest>) -> Result<Response<ListPeersResponse>, Status> {

        log::trace!("Received request: {:?}", request);

        let peers = self.peer_messaging_broker.list_peers().await;

        let peers = peers.into_iter()
            .map(From::from)
            .collect::<Vec<_>>();

        let reply = ListPeersResponse {
            peers,
        };

        Ok(Response::new(reply))
    }

    type OpenStream = Pin<Box<dyn Stream<Item = Result<Downstream, Status>> + Send>>;

    async fn open(&self, request: Request<Streaming<Upstream>>) -> Result<Response<Self::OpenStream>, Status> {

        let peer_id = PeerId::from(
            Uuid::parse_str(
                request.metadata()
                    .get("id")
                    .expect("Client should have sent an ID")
                    .to_str()
                    .expect("Client ID should be a valid string")
            ).expect("Client ID should be a valid UUID")
        );

        let mut inbound = request.into_inner();

        let (tx_inbound, rx_outbound) = self.peer_messaging_broker.open(peer_id).await;

        let peer_messaging_broker = Clone::clone(&self.peer_messaging_broker);

        tokio::spawn(async move {
            while let Some(result) = inbound.next().await {
                match result {
                    Ok(upstream) => {
                        log::trace!("Received message from client <{}>: {:?}", peer_id, upstream);
                        if let Some(message) = upstream.message {
                            tx_inbound.send(message).await.unwrap();
                        } else {
                            log::warn!("Ignoring empty message from client <{}>: {:?}", peer_id, upstream);
                        }
                    }
                    Err(status) => {
                        let _ = peer_messaging_broker.remove_peer(peer_id).await;
                        log::error!("Error: {:?}", status);
                    }
                }
            }
        });

        let outbound = ReceiverStream::new(rx_outbound)
            .map(|message| Ok(Downstream {
                message: Some(message)
            }));

        Ok(Response::new(Box::pin(outbound)))
    }
}
