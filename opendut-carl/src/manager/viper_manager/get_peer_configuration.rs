use opendut_model::cluster::{ClusterDescriptor, ClusterId};
use opendut_model::peer::PeerId;
use opendut_model::viper::{ViperTestId, ViperTestRunDescriptor};
use crate::resource::manager::{Resources, ResourcesStorageApi};
use crate::resource::manager::error::PersistenceError;

impl Resources<'_> {
    pub fn get_peer_id_for_test(&self, test_id: ViperTestId) -> Result<PeerId, GetPeerIdForTestError>{
        let test_run_descriptor = self.get::<ViperTestRunDescriptor>(test_id)
            .map_err(|error| GetPeerIdForTestError::Persistence { test_id, source: error })?
            .ok_or_else(|| GetPeerIdForTestError::TestRunDescriptorNotFound { test_id })?;

        let cluster_id = test_run_descriptor.cluster;

        let cluster_descriptor = self.get::<ClusterDescriptor>(cluster_id)
            .map_err(|error| GetPeerIdForTestError::Persistence { test_id, source: error })?
            .ok_or_else(|| GetPeerIdForTestError::ClusterDescriptorNotFound { cluster_id })?;

        let leader_id = cluster_descriptor.leader;

        Ok(leader_id)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum GetPeerIdForTestError {
    #[error("Peer id could not be fetched, because test run descriptor with the id <{test_id}> not found!")]
    TestRunDescriptorNotFound { test_id: ViperTestId },

    #[error("Peer id could not be fetched, because cluster descriptor with the id <{cluster_id}> not found!")]
    ClusterDescriptorNotFound { cluster_id: ClusterId },

    #[error("Peer id for test <{test_id}> could not be fetches, because of an error when accessing persistence!")]
    Persistence {
        test_id: ViperTestId,
        source: PersistenceError,
    },
}
