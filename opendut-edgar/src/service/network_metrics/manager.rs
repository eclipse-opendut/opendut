use std::{collections::HashMap, net::IpAddr, ops::Not, sync::{Arc, Mutex}, time::Duration};

use opendut_types::peer::PeerId;
use opendut_util::{project, settings::LoadedConfig};
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

        Ok(Arc::new(Mutex::new(Self {
            previous_spawn: None,
            options,
        })))
    }

    pub fn set_remote_peers(&mut self, remote_peers: HashMap<PeerId, IpAddr>) {

        if let Some(previous_spawn) = &mut self.previous_spawn {

            if previous_spawn.remote_peers == remote_peers {
                trace!("Received the same remote peers as with the previous PeerConfiguration. Ignoring.");
                return;
            }

            debug!("Terminating previous network metrics thread.");
            previous_spawn.abort_handle.abort();
        }

        if remote_peers.is_empty().not() {
            let NetworkMetricsOptions { ping_interval, target_bandwidth_kbit_per_second, rperf_backoff_max_elapsed_time } = self.options;

            let thread_handle = {
                let remote_peers = remote_peers.clone();

                tokio::spawn(async move {
                    debug!("Starting new network metrics thread for remote peers: {:?}", remote_peers);

                    super::ping::spawn_cluster_ping(remote_peers.clone(), ping_interval);

                    if project::is_running_in_development().not() {
                        let _ = super::rperf::server::exponential_backoff_launch_rperf_server(rperf_backoff_max_elapsed_time).await //ignore errors during startup of rperf server, as we do not want to crash EDGAR for this
                            .inspect_err(|cause| error!("Failed to start rperf server:\n  {cause}"));

                        super::rperf::client::launch_rperf_clients(remote_peers, target_bandwidth_kbit_per_second, rperf_backoff_max_elapsed_time).await;
                    }
                })
            };

            self.previous_spawn = Some(PreviousSpawn {
                remote_peers,
                abort_handle: thread_handle.abort_handle()
            });
        } else {
            self.previous_spawn = None;
        }
    }
}


#[derive(Clone, Debug)]
struct NetworkMetricsOptions {
    ping_interval: Duration,
    target_bandwidth_kbit_per_second: u64,
    rperf_backoff_max_elapsed_time: Duration,
}

#[derive(Debug)]
struct PreviousSpawn {
    remote_peers: HashMap<PeerId, IpAddr>,
    abort_handle: tokio::task::AbortHandle,
}
