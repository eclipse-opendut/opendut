use crate::resource::storage::ResourcesStorageApi;
use std::collections::HashSet;
use tracing::log::{error, trace};
use opendut_types::peer::configuration::{ParameterTargetState, PeerConfiguration, PeerConfigurationState};
use opendut_types::peer::PeerDescriptor;
use crate::manager::peer_messaging_broker::PeerMessagingBrokerRef;
use crate::resource::manager::{ResourceManagerRef, SubscriptionEvent};
use crate::resource::persistence::error::PersistenceResult;

pub(crate) async fn register(resource_manager: ResourceManagerRef, peer_messaging_broker: PeerMessagingBrokerRef) {
    disconnect_peer_when_removed(resource_manager.clone(), peer_messaging_broker.clone()).await;
    remove_absent_peer_configuration_parameters_that_are_absent(resource_manager).await;
}

pub async fn disconnect_peer_when_removed(resource_manager: ResourceManagerRef, peer_messaging_broker: PeerMessagingBrokerRef) {
    let mut peer_subscription = resource_manager.subscribe::<PeerDescriptor>().await;

    tokio::spawn(async move {
        loop {
            let peer_subscription = peer_subscription.receive().await;

            if let Ok(SubscriptionEvent::Removed { id: peer_id, ..  }) = peer_subscription {
                trace!("Peer <{peer_id}> was removed. Checking if there is an open peer connection.");
                let _ = peer_messaging_broker.remove_peer(peer_id).await
                    .inspect_err(|error| error!("Failed to remove peer <{peer_id}>: {error}"));
            }
        }
    });
}

/// Update the peer configuration by removing parameters that have been implemented and are no longer needed. 
/// These parameters, now considered redundant, serve no further purpose as their target is absent.
async fn remove_absent_peer_configuration_parameters_that_are_absent(resource_manager: ResourceManagerRef) {
    let mut peer_config_state_subscription = resource_manager.subscribe::<PeerConfigurationState>().await;

    tokio::spawn(async move {
        loop {
            let peer_subscription = peer_config_state_subscription.receive().await;

            if let Ok(SubscriptionEvent::Inserted { id, value }) = peer_subscription {
                
                let parameter_states_with_target_absent = value.parameter_states
                    .into_iter()
                    .filter(|p| matches!(p.state, ParameterTargetState::Absent))
                    .map(|p| p.id)
                    .collect::<HashSet<_>>();
                
                if !parameter_states_with_target_absent.is_empty() {
                    // ensure those parameters are removed from the peer configuration
                    let _result = resource_manager.resources_mut(async |resources| {
                        let peer_configuration: Option<PeerConfiguration> = resources.get::<PeerConfiguration>(id)?;
                        match peer_configuration {
                            Some(peer_configuration) => {
                                // Update peer configuration and remove parameters with target absent. Those parameters are now considered to be implemented by the peer and serve no purpose anymore.
                                let new_config = PeerConfiguration {
                                    device_interfaces: peer_configuration.device_interfaces.into_iter()
                                        .filter(|parameter| !parameter_states_with_target_absent.contains(&parameter.id)).collect(),
                                    ethernet_bridges: peer_configuration.ethernet_bridges.into_iter()
                                        .filter(|parameter| !parameter_states_with_target_absent.contains(&parameter.id)).collect(),
                                    executors: peer_configuration.executors.into_iter()
                                        .filter(|parameter| !parameter_states_with_target_absent.contains(&parameter.id)).collect(),
                                    gre_interfaces: peer_configuration.gre_interfaces.into_iter()
                                        .filter(|parameter| !parameter_states_with_target_absent.contains(&parameter.id)).collect(),
                                    joined_interfaces: peer_configuration.joined_interfaces.into_iter()
                                        .filter(|parameter| !parameter_states_with_target_absent.contains(&parameter.id)).collect(),
                                    remote_peer_connection_checks: peer_configuration.remote_peer_connection_checks.into_iter()
                                        .filter(|parameter| !parameter_states_with_target_absent.contains(&parameter.id)).collect(),
                                };
                                resources.insert(id, new_config)
                            }
                            None => {
                                // PeerConfiguration does not exist, this is an unexpected error
                                PersistenceResult::Ok(())
                            }
                        }
                    }).await
                        .inspect_err(|cause| {
                            error!("Failed to remove absent peer configuration parameters for peer <{id}>: {cause}");
                        });
                }
            }
        }
    });
}
