use std::collections::HashMap;
use std::net::IpAddr;
use async_trait::async_trait;
use tracing::debug;
use opendut_types::peer::PeerId;
use crate::common::task::{Success, Task, TaskStateFulfilled};
use crate::service::network_metrics::manager::NetworkMetricsManagerRef;

pub struct SetupClusterMetrics {
    pub remote_peers: HashMap<PeerId, IpAddr>,
    pub metrics_manager: NetworkMetricsManagerRef,
}

#[async_trait]
impl Task for SetupClusterMetrics {
    fn description(&self) -> String {
        String::from("Setup cluster metrics")
    }

    async fn check_present(&self) -> anyhow::Result<TaskStateFulfilled> {
        Ok(TaskStateFulfilled::Unchecked)
    }

    async fn make_present(&self) -> anyhow::Result<Success> {
        debug!("Setting up cluster metrics.");

        self.metrics_manager.lock().await
            .set_remote_peers(self.remote_peers.clone()).await;

        Ok(Success::default())
    }
}

//TODO impl TaskAbsent
