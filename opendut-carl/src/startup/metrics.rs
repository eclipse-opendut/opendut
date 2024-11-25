use opentelemetry::global;
use tracing::trace;
use opendut_types::cluster::{ClusterConfiguration, ClusterDeployment};
use opendut_types::peer::PeerDescriptor;
use opendut_types::peer::state::PeerState;
use crate::resources::manager::ResourcesManagerRef;

pub fn initialize_metrics_collection(
    resources_manager: ResourcesManagerRef,
) {
    let meter = global::meter(opendut_util::telemetry::DEFAULT_METER_NAME);

    {
        let resources_manager = resources_manager.clone();
        meter.u64_observable_gauge("deployed_clusters")
            .with_callback(move |observer| {
                let result = futures::executor::block_on(
                    resources_manager.list::<ClusterDeployment>()
                );

                match result {
                    Ok(deployed_clusters) => observer.observe(deployed_clusters.len() as u64, &[]),
                    Err(cause) => trace!("Error while loading metrics information from ResourcesManager:\n  {cause}")
                }
            })
            .build();
    }

    {
        let resources_manager = resources_manager.clone();
        meter.u64_observable_gauge("configured_clusters")
            .with_callback(move |observer| {
                let result = futures::executor::block_on(
                    resources_manager.list::<ClusterConfiguration>()
                );

                match result {
                    Ok(configured_clusters) => observer.observe(configured_clusters.len() as u64, &[]),
                    Err(cause) => trace!("Error while loading metrics information from ResourcesManager:\n  {cause}")
                }
            })
            .build();
    }

    {
        let resources_manager = resources_manager.clone();
        meter.u64_observable_gauge("registered_peers")
            .with_callback(move |observer| {
                let result = futures::executor::block_on(
                    resources_manager.list::<PeerDescriptor>()
                );

                match result {
                    Ok(registered_peers) => observer.observe(registered_peers.len() as u64, &[]),
                    Err(cause) => trace!("Error while loading metrics information from ResourcesManager:\n  {cause}")
                }
            })
            .build();
    }

    {
        let resources_manager = resources_manager.clone();
        meter.u64_observable_gauge("connected_peers")
            .with_callback(move |observer| {
                let result = futures::executor::block_on(
                    resources_manager.list::<PeerState>()
                );

                match result {
                    Ok(peer_states) => {
                        let online_peers = peer_states.into_iter()
                            .filter(|state| matches!(state, PeerState::Up { .. }))
                            .collect::<Vec<_>>();

                        observer.observe(online_peers.len() as u64, &[])
                    }
                    Err(cause) => trace!("Error while loading metrics information from ResourcesManager:\n  {cause}")
                }
            })
            .build();
    }
}
