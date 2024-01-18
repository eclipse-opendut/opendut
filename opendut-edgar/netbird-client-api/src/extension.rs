use std::fmt::Debug;
use std::net::Ipv4Addr;
use std::str::FromStr;

use crate::proto::daemon::LocalPeerState;

pub trait LocalPeerStateExtension {
    fn local_ip(&self) -> Result<Ipv4Addr, LocalIpParseError>;
}

impl LocalPeerStateExtension for LocalPeerState {
    fn local_ip(&self) -> Result<Ipv4Addr, LocalIpParseError> {
        let local_ip = &self.ip;

        let local_ip = local_ip.split('/').next() //strip CIDR mask
            .ok_or(LocalIpParseError { message: format!("Iterator.split() should always return a first element. Did not do so when stripping CIDR mask off of local IP '{local_ip}'.") })?;

        let local_ip = Ipv4Addr::from_str(local_ip)
            .map_err(|cause| LocalIpParseError { message: format!("Local IP returned by NetBird '{local_ip}' could not be parsed: {cause}") })?;

        Ok(local_ip)
    }
}


#[derive(Debug, thiserror::Error)]
#[error("Local IP parse error: {}", self.message)]
pub struct LocalIpParseError { message: String }
