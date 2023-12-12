use opendut_types::cluster::{ClusterConfiguration, ClusterId, ClusterName};
use opendut_types::peer::PeerId;

use crate::clusters::configurator::components::DeviceSelection;
use crate::components::UserInputValue;

#[derive(thiserror::Error, Clone, Debug)]
pub enum ClusterMisconfiguration { // TODO: Maybe replace with IllegalClusterConfiguration from opendut-types.
    #[error("Invalid cluster name")]
    InvalidClusterName,

    #[error("Invalid cluster name")]
    InvalidDeviceSelection
}

#[derive(Clone, Debug)]
pub struct UserClusterConfiguration {
    pub id: ClusterId,
    pub name: UserInputValue,
    pub devices: DeviceSelection,
}

impl UserClusterConfiguration {

    pub fn is_valid(&self) -> bool {
        self.name.is_right()
            && self.devices.is_right()
    }
}

impl TryFrom<UserClusterConfiguration> for ClusterConfiguration {

    type Error = ClusterMisconfiguration;

    fn try_from(configuration: UserClusterConfiguration) -> Result<Self, Self::Error> {
        let name = configuration.name
            .right_ok_or(ClusterMisconfiguration::InvalidClusterName)
            .and_then(|name| ClusterName::try_from(name)
                .map_err(|_| ClusterMisconfiguration::InvalidClusterName))?;
        let devices = configuration.devices.right_ok_or(ClusterMisconfiguration::InvalidDeviceSelection)?;
        Ok(ClusterConfiguration {
            id: configuration.id,
            name,
            leader: PeerId::default(), // TODO: Insert the leader's PeerId here.
            devices,
        })
    }
}
