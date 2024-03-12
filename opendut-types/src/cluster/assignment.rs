use std::net::IpAddr;
use crate::cluster::ClusterId;
use crate::peer::PeerId;
use crate::util::net::NetworkInterfaceDescriptor;
use crate::util::Port;


#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClusterAssignment {
    pub id: ClusterId,
    pub leader: PeerId,
    pub assignments: Vec<PeerClusterAssignment>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PeerClusterAssignment {
    pub peer_id: PeerId,
    pub vpn_address: IpAddr,
    pub can_server_port: Port,
    pub device_interfaces: Vec<NetworkInterfaceDescriptor>,
}
