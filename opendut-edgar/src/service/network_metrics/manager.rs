use std::{collections::HashMap, net::IpAddr, ops::Not, sync::Arc, time::Duration};

use opendut_model::peer::PeerId;
use opendut_util::{project, settings::LoadedConfig};
use tokio::sync::Mutex;
use tracing::{debug, error, trace};


pub type NetworkMetricsManagerRef = Arc<Mutex<NetworkMetricsManager>>;

#[derive(Debug)]
pub struct NetworkMetricsManager {
    previous_spawn: Option<PreviousSpawn>,
    options: NetworkMetricsOptions,
}

impl NetworkMetricsManager {
    pub fn load(settings: &LoadedConfig) -> anyhow::Result<NetworkMetricsManagerRef> {

        let ping_interval = Duration::from_millis(settings.config.get::<u64>("opentelemetry.metrics.cluster.ping.interval.ms")?);
        let target_bandwidth_kbit_per_second = settings.config.get::<u64>("opentelemetry.metrics.cluster.target.bandwidth.kilobit.per.second")?;
        let rperf_backoff_max_elapsed_time = Duration::from_millis(settings.config.get::<u64>("opentelemetry.metrics.cluster.rperf.backoff.max.elapsed.time.ms")?);

        let options = NetworkMetricsOptions {
            ping_interval,
            target_bandwidth_kbit_per_second,
            rperf_backoff_max_elapsed_time,
        };

        Ok(Self::new(options))
    }

    pub fn new(options: NetworkMetricsOptions) -> NetworkMetricsManagerRef {
        Arc::new(Mutex::new(Self {
            previous_spawn: None,
            options,
        }))
    }

    pub async fn set_remote_peers(&mut self, remote_peers: HashMap<PeerId, IpAddr>) {

        if let Some(previous_spawn) = &mut self.previous_spawn {

            if previous_spawn.remote_peers == remote_peers {
                trace!("Received the same remote peers as with the previous PeerConfiguration. Not restarting the network metrics thread.");
                return;
            }

            debug!("Terminating previous network metrics thread.");
            previous_spawn.spawner.lock().await
                .shutdown().await;
        }

        if remote_peers.is_empty().not() {
            let NetworkMetricsOptions { ping_interval, target_bandwidth_kbit_per_second, rperf_backoff_max_elapsed_time } = self.options;

            let spawner = {
                let remote_peers = remote_peers.clone();

                debug!(
                    "Starting new network metrics thread for remote peers: {:?}",
                    remote_peers.iter()
                        .map(|(key, value)| (key.uuid, value))
                        .collect::<HashMap<_, _>>()
                );

                let spawner: Spawner = Arc::new(Mutex::new(tokio::task::JoinSet::new()));

                spawner.lock().await
                    .spawn(super::ping::spawn_cluster_ping(remote_peers.clone(), ping_interval));

                if project::is_running_in_development().not() {
                    let _ = super::rperf::server::exponential_backoff_launch_rperf_server(spawner.clone(), rperf_backoff_max_elapsed_time).await //ignore errors during startup of rperf server, as we do not want to crash EDGAR for this
                        .inspect_err(|cause| error!("Failed to start rperf server:\n  {cause}"));

                    super::rperf::client::launch_rperf_clients(remote_peers, spawner.clone(), target_bandwidth_kbit_per_second, rperf_backoff_max_elapsed_time).await;
                }

                spawner
            };

            self.previous_spawn = Some(PreviousSpawn {
                remote_peers,
                spawner,
            });
        } else {
            self.previous_spawn = None;
        }
    }
}


pub type Spawner = Arc<Mutex<tokio::task::JoinSet<()>>>;


#[derive(Clone, Debug)]
pub struct NetworkMetricsOptions {
    ping_interval: Duration,
    target_bandwidth_kbit_per_second: u64,
    rperf_backoff_max_elapsed_time: Duration,
}

impl Default for NetworkMetricsOptions {
    fn default() -> Self {
        Self {
            ping_interval: Duration::from_secs(30),
            target_bandwidth_kbit_per_second: 100_000, // 100 Mbit/s
            rperf_backoff_max_elapsed_time: Duration::from_secs(120),
        }
    }
}

#[derive(Debug)]
struct PreviousSpawn {
    remote_peers: HashMap<PeerId, IpAddr>,
    spawner: Spawner,
}
