use opendut_types::peer::{PeerDescriptor, PeerId, PeerName};
use opendut_types::topology::Topology;

use crate::components::UserInputValue;

#[derive(thiserror::Error, Clone, Debug)]
pub enum PeerMisconfiguration { // TODO: Maybe introduce a IllegalPeerDescriptor to opendut-types.
    #[error("Invalid peer name")]
    InvalidPeerName,
}

#[derive(Clone, Debug)]
pub struct UserPeerConfiguration {
    pub id: PeerId,
    pub name: UserInputValue,
    pub is_new: bool,
}

impl UserPeerConfiguration {
    pub fn is_valid(&self) -> bool {
        self.name.is_right()
    }
}

impl TryFrom<UserPeerConfiguration> for PeerDescriptor {

    type Error = PeerMisconfiguration;

    fn try_from(configuration: UserPeerConfiguration) -> Result<Self, Self::Error> {
        let name = configuration.name
            .right_ok_or(PeerMisconfiguration::InvalidPeerName)
            .and_then(|name| PeerName::try_from(name)
                .map_err(|_| PeerMisconfiguration::InvalidPeerName))?;
        Ok(PeerDescriptor {
            id: configuration.id,
            name,
            topology: Topology::default(),
        })
    }
}
