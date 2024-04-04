use crate::proto::{ConversionError, ConversionErrorBuilder};

include!(concat!(env!("OUT_DIR"), "/opendut.types.peer.configuration.rs"));

impl From<crate::peer::configuration::PeerConfiguration> for PeerConfiguration {
    fn from(value: crate::peer::configuration::PeerConfiguration) -> Self {
        Self {
            executors: Some(value.executors.into()),
            cluster_assignment: value.cluster_assignment.map(|assignment| assignment.into()),
            network: Some(value.network.into())
        }
    }
}

impl TryFrom<PeerConfiguration> for crate::peer::configuration::PeerConfiguration {
    type Error = ConversionError;

    fn try_from(value: PeerConfiguration) -> Result<Self, Self::Error> {
        type ErrorBuilder = ConversionErrorBuilder<PeerConfiguration, crate::peer::configuration::PeerConfiguration>;

        let executors = value.executors
            .ok_or(ErrorBuilder::field_not_set("executors"))?
            .try_into()?;

        let cluster_assignment = value.cluster_assignment
            .map(TryInto::try_into)
            .transpose()?;

        let network = value.network
            .ok_or(ErrorBuilder::field_not_set("network"))?
            .try_into()?;

        Ok(crate::peer::configuration::PeerConfiguration {
            executors,
            cluster_assignment,
            network
        })
    }
}

impl From<crate::peer::configuration::PeerNetworkConfiguration> for PeerNetworkConfiguration {
    fn from(value: crate::peer::configuration::PeerNetworkConfiguration) -> Self {
        Self {
            bridge_name: Some(value.bridge_name.into())
        }
    }
}

impl TryFrom<PeerNetworkConfiguration> for crate::peer::configuration::PeerNetworkConfiguration {
    type Error = ConversionError;

    fn try_from(value: PeerNetworkConfiguration) -> Result<Self, Self::Error> {
        type ErrorBuilder = ConversionErrorBuilder<PeerNetworkConfiguration, crate::peer::configuration::PeerNetworkConfiguration>;

        let bridge_name = value.bridge_name
            .ok_or(ErrorBuilder::field_not_set("bridge_name"))?
            .try_into()?;

        Ok(crate::peer::configuration::PeerNetworkConfiguration {
            bridge_name,
        })
    }
}
