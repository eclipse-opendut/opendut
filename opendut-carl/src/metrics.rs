use opentelemetry::global;
use tracing::trace;
use opendut_types::cluster::{ClusterConfiguration, ClusterDeployment};
use opendut_types::peer::PeerDescriptor;
use opendut_types::peer::state::PeerState;

use crate::resources::manager::ResourcesManagerRef;


pub fn initialize_metrics_collection(
    resources_manager: ResourcesManagerRef,
) {
    let meter = global::meter(opendut_util::logging::DEFAULT_METER_NAME);

    let deployed_clusters_gauge = meter.u64_observable_gauge("deployed_clusters").init();
    let configured_clusters_gauge = meter.u64_observable_gauge("configured_clusters").init();
    let registered_peers_gauge = meter.u64_observable_gauge("registered_peers").init();
    let connected_peers_gauge = meter.u64_observable_gauge("connected_peers").init();
    meter.register_callback(&[deployed_clusters_gauge.as_any(), configured_clusters_gauge.as_any(), registered_peers_gauge.as_any(), connected_peers_gauge.as_any()], move |observer| {

        let metrics = futures::executor::block_on(
            resources_manager.resources(|resources| {
                Metrics {
                    deployed_clusters: resources.iter::<ClusterDeployment>().cloned().collect::<Vec<_>>(),
                    configured_clusters: resources.iter::<ClusterConfiguration>().cloned().collect::<Vec<_>>(),
                    registered_peers: resources.iter::<PeerDescriptor>().cloned().collect::<Vec<_>>(),
                    connected_peers: {
                        let registered_peers = resources.iter::<PeerState>().cloned().collect::<Vec<PeerState>>();
                        let mut online_peers: Vec<PeerState> = vec![];
                        registered_peers.iter().for_each(|state| {
                            if let PeerState::Up { .. } = state { online_peers.push(state.clone()) }
                        });
                        online_peers
                    },
                }
            })
        );
        match ObservableMetrics::try_from(metrics) {
            Ok(metrics) => {
                observer.observe_u64(&deployed_clusters_gauge, metrics.deployed_clusters, &[]);
                observer.observe_u64(&configured_clusters_gauge, metrics.configured_clusters, &[]);
                observer.observe_u64(&registered_peers_gauge, metrics.registered_peers, &[]);
                observer.observe_u64(&connected_peers_gauge, metrics.connected_peers, &[]);
            }
            Err(cause) => trace!("Error while converting metrics into observable format:\n  {cause}")
        };

    }).expect("could not register metrics collection callback for CARL");
}

struct Metrics {
    deployed_clusters: Vec<ClusterDeployment>,
    configured_clusters: Vec<ClusterConfiguration>,
    registered_peers: Vec<PeerDescriptor>,
    connected_peers: Vec<PeerState>,
}

struct ObservableMetrics {
    deployed_clusters: u64,
    configured_clusters: u64,
    registered_peers: u64,
    connected_peers: u64,
}
impl TryFrom<Metrics> for ObservableMetrics {
    type Error = anyhow::Error;

    fn try_from(metrics: Metrics) -> Result<Self, Self::Error> {
        Ok(Self {
            deployed_clusters: u64::try_from(metrics.deployed_clusters.len())?,
            configured_clusters: u64::try_from(metrics.configured_clusters.len())?,
            registered_peers: u64::try_from(metrics.registered_peers.len())?,
            connected_peers: u64::try_from(metrics.connected_peers.len())?,
        })
    }
}

