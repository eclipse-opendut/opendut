use crate::proto::{ConversionError, ConversionErrorBuilder};

include!(concat!(env!("OUT_DIR"), "/opendut.types.peer.configuration.rs"));

impl From<crate::peer::configuration::PeerConfiguration> for PeerConfiguration {
    fn from(value: crate::peer::configuration::PeerConfiguration) -> Self {
        Self {
            executors: Some(value.executors.into()),
            cluster_assignment: value.cluster_assignment.map(|assignment| assignment.into()),
        }
    }
}

impl TryFrom<PeerConfiguration> for crate::peer::configuration::PeerConfiguration {
    type Error = ConversionError;

    fn try_from(value: PeerConfiguration) -> Result<Self, Self::Error> {
        type ErrorBuilder = ConversionErrorBuilder<PeerConfiguration, crate::peer::configuration::PeerConfiguration>;

        let executors = value.executors
            .ok_or(ErrorBuilder::new("Executor not set"))?
            .try_into()?;

        let cluster_assignment = value.cluster_assignment
            .map(TryInto::try_into)
            .transpose()?;

        Ok(crate::peer::configuration::PeerConfiguration {
            executors,
            cluster_assignment
        })
    }
}
