use opendut_types::cluster::{ClusterConfiguration, ClusterId, ClusterName};

use crate::clusters::configurator::components::{DeviceSelection, LeaderSelection};
use crate::components::UserInputValue;

#[derive(thiserror::Error, Clone, Debug)]
#[allow(clippy::enum_variant_names)] // "all variants have the same prefix: `Invalid`"
pub enum ClusterMisconfiguration { // TODO: Maybe replace with IllegalClusterConfiguration from opendut-types.
    #[error("Invalid cluster name")]
    InvalidClusterName,

    #[error("Invalid device selection")]
    InvalidDeviceSelection,

    #[error("Invalid leader selection")]
    InvalidLeaderSelection
}

#[derive(Clone, Debug)]
pub struct UserClusterConfiguration {
    pub id: ClusterId,
    pub name: UserInputValue,
    pub devices: DeviceSelection,
    pub leader: LeaderSelection,
}

impl UserClusterConfiguration {

    pub fn is_valid(&self) -> bool {
        self.name.is_right()
            && self.devices.is_right()
            && self.leader.is_right()
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
        let leader = configuration.leader.right_ok_or(ClusterMisconfiguration::InvalidLeaderSelection)?;
        Ok(ClusterConfiguration {
            id: configuration.id,
            name,
            leader,
            devices,
        })
    }
}
