use crate::service::can::can_manager::CanManagerRef;
use crate::service::network_interface::manager::NetworkInterfaceManagerRef;
use crate::service::test_execution::executor_manager::ExecutorManagerRef;
use crate::service::viper_run_manager::ViperRunManagerRef;
use opendut_model::peer::configuration::{EdgePeerConfigurationParameterState, EdgePeerConfigurationState, ParameterVariant, PeerConfiguration};

use std::fmt::Formatter;
use std::sync::Arc;
use serde::Serialize;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error};
use opendut_model::format::DebugJsonDisplay;
use crate::service::tasks::runner;
use crate::service::tasks::runner::service_runner::CollectedResult;
use super::network_metrics::manager::NetworkMetricsManagerRef;

#[derive(Debug)]
pub struct ApplyPeerConfigurationParams {
    pub peer_configuration: PeerConfiguration,
    pub network_interface_management: NetworkInterfaceManagement,
    pub executor_manager: ExecutorManagerRef,
    pub metrics_manager: NetworkMetricsManagerRef,
    pub viper_run_manager: ViperRunManagerRef,
}
#[derive(Clone)]
pub enum NetworkInterfaceManagement {
    Enabled { network_interface_manager: NetworkInterfaceManagerRef, can_manager: CanManagerRef },
    Disabled,
}
impl std::fmt::Debug for NetworkInterfaceManagement {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            NetworkInterfaceManagement::Enabled { .. } => writeln!(f, "Enabled"),
            NetworkInterfaceManagement::Disabled => writeln!(f, "Disabled"),
        }
    }
}

#[derive(Debug, Serialize)]
struct EdgePeerConfigurationParameterDifference {
    expected_parameter: ParameterVariant,
    detected_state: EdgePeerConfigurationParameterState,
}

pub async fn spawn_peer_configurations_handler(
    rx_peer_configuration: mpsc::Receiver<ApplyPeerConfigurationParams>,
    tx_peer_configuration_state: mpsc::Sender<EdgePeerConfigurationState>,
    connect_cancel: CancellationToken,
) -> anyhow::Result<JoinHandle<()>> {
    let handle = tokio::spawn(async move {
        tokio::select! {
            _ = spawn_peer_configuration_handler_loop(rx_peer_configuration, tx_peer_configuration_state) => {
                debug!("Peer configuration handling received end of stream.");
            }
            _ = connect_cancel.cancelled() => {
                debug!("Peer configuration handling was explicitly cancelled.");
            }
        }
    });
    Ok(handle)
}

async fn spawn_peer_configuration_handler_loop(
    mut rx_peer_configuration: mpsc::Receiver<ApplyPeerConfigurationParams>,
    tx_peer_configuration_state: mpsc::Sender<EdgePeerConfigurationState>,
) {
    while let Some(apply_peer_configuration_params) = rx_peer_configuration.recv().await {
        let given_peer_configuration_parameters = apply_peer_configuration_params.peer_configuration.all_parameters();

        let result = apply_peer_configuration(apply_peer_configuration_params).await;
        let state = EdgePeerConfigurationState::from(result);
        debug!("Sending peer configuration state to CARL: {}", state.to_debug_json());
        let mut failed = vec![];
        let mut unknown = vec![];
        for param_state in &state.parameter_states {
            if !param_state.detected_state.is_successful() {
                let parameter = given_peer_configuration_parameters.get(&param_state.id);
                match parameter {
                    None => {
                        unknown.push(param_state.clone());
                    }
                    Some(parameter) => {
                        failed.push(EdgePeerConfigurationParameterDifference { expected_parameter: parameter.clone(), detected_state: param_state.clone()  });
                    }
                }
            }
        }
        if !failed.is_empty() {
            error!("Some parameters failed to apply: {}", failed.to_debug_json());
        }
        if !unknown.is_empty() {
            error!("Some unknown parameters were reported in the state: {}", unknown.to_debug_json());
        }
        let _ = tx_peer_configuration_state.send(state).await
            .inspect_err(|cause| error!("Failed to send peer configuration state to CARL: {cause}"));
    }
}


#[tracing::instrument(skip_all)]
async fn apply_peer_configuration(params: ApplyPeerConfigurationParams) -> CollectedResult {
    let ApplyPeerConfigurationParams { 
        peer_configuration,
        network_interface_management, 
        executor_manager,
        metrics_manager,
        viper_run_manager,
    } = params;

    let resolver = runner::task_resolver::ServiceTaskResolver::new(
        peer_configuration.clone(),
        network_interface_management.clone(),
        Arc::clone(&metrics_manager),
        Arc::clone(&viper_run_manager),
    );
    let result = runner::service_runner::run_tasks(peer_configuration.clone(), resolver).await;
    if result.success {
        debug!("Peer configuration tasks executed successfully: {}", result.to_debug_json());
    } else {
        error!("Failed to apply peer configuration tasks. Collected result is: {}", result.to_debug_json());
        return result;
    }

    {
        let mut executor_manager = executor_manager.lock().await;
        executor_manager.terminate_executors();
        executor_manager.create_new_executors(peer_configuration.executors);
    }

    debug!("Peer configuration has been successfully applied.");
    result
}
