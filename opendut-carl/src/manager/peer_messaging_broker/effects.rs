use crate::manager::peer_messaging_broker::PeerMessagingBrokerRef;
use crate::resource::manager::{ResourceManagerRef, SubscriptionEvent};
use crate::resource::persistence::error::PersistenceResult;
use crate::resource::storage::ResourcesStorageApi;
use opendut_model::peer::configuration::{ParameterTargetState, PeerConfiguration, EdgePeerConfigurationState};
use opendut_model::peer::PeerDescriptor;
use std::collections::HashSet;
use tracing::{debug, error, trace, warn};

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
    let mut peer_config_state_subscription = resource_manager.subscribe::<EdgePeerConfigurationState>().await;

    tokio::spawn(async move {
        loop {
            let peer_subscription = peer_config_state_subscription.receive().await;

            if let Ok(SubscriptionEvent::Inserted { id, value }) = peer_subscription {
                
                let parameter_ids_with_target_absent = value.parameter_states
                    .into_iter()
                    .filter(|p| matches!(p.state, ParameterTargetState::Absent))
                    .map(|p| p.id)
                    .collect::<HashSet<_>>();
                
                if !parameter_ids_with_target_absent.is_empty() {
                    // ensure those parameters are removed from the peer configuration
                    let _result = resource_manager.resources_mut(async |resources| {
                        let peer_configuration: Option<PeerConfiguration> = resources.get::<PeerConfiguration>(id)?;
                        if let Some(mut peer_configuration) = peer_configuration {
                            debug!("Removing absent peer configuration parameters for peer <{id}>: {parameter_ids_with_target_absent:?}");
                            peer_configuration.remove_parameters(&parameter_ids_with_target_absent);
                            resources.insert(id, peer_configuration)
                        } else {
                            warn!("Could not find peer configuration for peer <{id}> to remove absent parameters. This might be due to the peer being removed before receiving an update from the peer.");
                            PersistenceResult::Ok(())
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
