use std::net::IpAddr;
use std::ops::Not;
use std::pin::Pin;
use std::str::FromStr;

use futures::StreamExt;
use tokio_stream::Stream;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status, Streaming};
use tonic::metadata::MetadataMap;
use tonic_web::CorsGrpcWeb;
use uuid::Uuid;

use opendut_carl_api::proto::services::peer_messaging_broker::{Downstream, ListPeersRequest, ListPeersResponse, Upstream};
use opendut_carl_api::proto::services::peer_messaging_broker::peer_messaging_broker_server::PeerMessagingBrokerServer;
use opendut_carl_api::proto::services::peer_messaging_broker::upstream;
use opendut_types::peer::PeerId;

use crate::peer::broker::PeerMessagingBrokerRef;

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
    #[tracing::instrument(skip(self, request), level="trace")]
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

    #[tracing::instrument(skip(self, request), level="trace")]
    async fn open(&self, request: Request<Streaming<Upstream>>) -> Result<Response<Self::OpenStream>, Status> {

        let peer_id = extract_peer_id(request.metadata())
            .map_err(|message| {
                log::warn!("Error while parsing PeerId from client request: {message}");
                Status::invalid_argument(message)
            })?;

        let remote_host = extract_remote_host(request.metadata())
            .map_err(|message| {
                log::warn!("Error while parsing remote host address from client request: {message}");
                Status::invalid_argument(message)
            })?;


        let (tx_inbound, rx_outbound) = self.peer_messaging_broker.open(peer_id, remote_host).await;

        let peer_messaging_broker = Clone::clone(&self.peer_messaging_broker);

        let mut inbound = request.into_inner();
        tokio::spawn(async move {
            while let Some(result) = inbound.next().await {
                match result {
                    Ok(upstream) => {
                        if let Some(message) = upstream.message {
                            if matches!(message, upstream::Message::Ping(_)).not() {
                                log::trace!("Received message from client <{}>: {:?}", peer_id, message);
                            }
                            tx_inbound.send(message).await.unwrap();
                        } else {
                            log::warn!("Ignoring empty message from client <{}>: {:?}", peer_id, upstream);
                        }
                    }
                    Err(status) => {
                        log::error!("Error: {:?}", status);
                    }
                }
            }

            if let Err(cause) = peer_messaging_broker.remove_peer(peer_id).await {
                log::error!("Failed to removed peer <{peer_id}>:\n  {cause}");
            }
            else {
                log::info!("Removed peer <{peer_id}>.");
            };
        });

        let outbound = ReceiverStream::new(rx_outbound)
            .map(|downstream| {
                Ok(downstream)
            });

        Ok(Response::new(Box::pin(outbound)))
    }
}


fn extract_peer_id(metadata: &MetadataMap) -> Result<PeerId, UserError> {
    let peer_id = PeerId::from(
        Uuid::parse_str(
            metadata
                .get("id")
                .ok_or("Client should have sent an ID")?
                .to_str()
                .map_err(|_| "Client ID should be a valid string")?
        ).map_err(|_| "Client ID should be a valid UUID")?
    );
    Ok(peer_id)
}

fn extract_remote_host(metadata: &MetadataMap) -> Result<IpAddr, UserError> {
    let remote_host = IpAddr::from_str(
        metadata
            .get("remote-host")
            .ok_or("Client should have sent a remote host address")?
            .to_str()
            .map_err(|_| "Remote host address should be a valid string")?
    ).map_err(|_| "Remote host address should be a valid IP address")?;

    Ok(remote_host)
}


type UserError = String;
