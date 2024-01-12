use std::net::IpAddr;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PeerId(pub String);


#[derive(Deserialize)]
pub struct Peer {
    pub id: PeerId,
    pub ip: IpAddr,
    //Further fields omitted: https://docs.netbird.io/api/resources/peers#retrieve-a-peer
}
