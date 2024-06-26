use std::path::PathBuf;

use opendut_types::peer::executor::ExecutorDescriptor;
use serde::{Deserialize, Serialize};

use opendut_carl_api::carl::CarlClient;
use opendut_types::peer::PeerId;

use crate::{CreateOutputFormat, DescribeOutputFormat};

/// Create a container executor using a JSON-formatted configuration file
#[derive(clap::Parser)]
pub struct ApplyContainerExecutorCli {
    ///Path to the JSON-formatted executor configuration file
    config_file: PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct ExecutorConfiguration {
    peer_id: PeerId,
    #[serde(flatten)]
    executor_descriptor: ExecutorDescriptor,
}

impl ApplyContainerExecutorCli {
    #[allow(clippy::too_many_arguments)]
    pub async fn execute(self, carl: &mut CarlClient, output: CreateOutputFormat) -> crate::Result<()> {

        let config_str = std::fs::read_to_string(&self.config_file)
            .map_err(|cause| format!("Failed to open file '{}': {}", self.config_file.display(), cause))?;

        let executor_configuration: ExecutorConfiguration = serde_json::from_str(&config_str)
            .map_err(|cause| format!("Failed to parse '{}' as executor configuration: {}", self.config_file.display(), cause))?;

        let peer_id = executor_configuration.peer_id;
        let executor_descriptor = executor_configuration.executor_descriptor;

        let mut peer_descriptor = carl.peers.get_peer_descriptor(peer_id).await
            .map_err(|_| format!("Failed to get peer with ID <{}>.", peer_id))?;

        peer_descriptor.executors.executors.push(executor_descriptor);

        carl.peers.store_peer_descriptor(Clone::clone(&peer_descriptor)).await
            .map_err(|error| format!("Failed to update peer <{}>.\n  {}", peer_id, error))?;
        let output_format = DescribeOutputFormat::from(output);
        crate::commands::peer::describe::render_peer_descriptor(peer_descriptor, output_format);

        Ok(())
    }
}
