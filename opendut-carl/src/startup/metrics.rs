use opentelemetry::global;
use tracing::trace;
use opendut_model::cluster::{ClusterDescriptor, ClusterDeployment};
use opendut_model::peer::PeerDescriptor;
use opendut_model::peer::state::PeerConnectionState;
use crate::resource::manager::ResourceManagerRef;

pub fn initialize_metrics_collection(
    resource_manager: ResourceManagerRef,
) {
    let meter = global::meter(opendut_util::telemetry::DEFAULT_METER_NAME);

    {
        let resource_manager = resource_manager.clone();
        meter.u64_observable_gauge("deployed_clusters")
            .with_callback(move |observer| {
                let result = futures::executor::block_on(
                    resource_manager.list::<ClusterDeployment>()
                );

                match result {
                    Ok(deployed_clusters) => observer.observe(deployed_clusters.len() as u64, &[]),
                    Err(cause) => trace!("Error while loading metrics information from ResourceManager:\n  {cause}")
                }
            })
            .build();
    }

    {
        let resource_manager = resource_manager.clone();
        meter.u64_observable_gauge("configured_clusters")
            .with_callback(move |observer| {
                let result = futures::executor::block_on(
                    resource_manager.list::<ClusterDescriptor>()
                );

                match result {
                    Ok(configured_clusters) => observer.observe(configured_clusters.len() as u64, &[]),
                    Err(cause) => trace!("Error while loading metrics information from ResourceManager:\n  {cause}")
                }
            })
            .build();
    }

    {
        let resource_manager = resource_manager.clone();
        meter.u64_observable_gauge("registered_peers")
            .with_callback(move |observer| {
                let result = futures::executor::block_on(
                    resource_manager.list::<PeerDescriptor>()
                );

                match result {
                    Ok(registered_peers) => observer.observe(registered_peers.len() as u64, &[]),
                    Err(cause) => trace!("Error while loading metrics information from ResourceManager:\n  {cause}")
                }
            })
            .build();
    }

    {
        let resource_manager = resource_manager.clone();
        meter.u64_observable_gauge("connected_peers")
            .with_callback(move |observer| {
                let result = futures::executor::block_on(
                    resource_manager.list::<PeerConnectionState>()
                );

                match result {
                    Ok(peer_connection_states) => {
                        let online_peers = peer_connection_states.into_iter()
                            .filter(|(_, connection)| matches!(connection, PeerConnectionState::Online { .. }))
                            .collect::<Vec<_>>();

                        observer.observe(online_peers.len() as u64, &[]);
                    }
                    Err(cause) => trace!("Error while loading metrics information from ResourceManager:\n  {cause}")
                }
            })
            .build();
    }
}
