use std::collections::HashMap;
use crate::common::task::dependency::PeerConfigurationDependencyResolver;
use crate::common::task::{Success, TaskAbsent, TaskStateFulfilled};
use crate::service::tasks;
use opendut_types::peer::configuration::{ParameterTarget, ParameterVariant, PeerConfiguration};
use crate::common::task::runner::{Outcome, TaskExecutionError};
use crate::service::network_metrics::manager::NetworkMetricsManagerRef;
use crate::service::peer_configuration::NetworkInterfaceManagement;

fn resolve_tasks(parameter: &ParameterVariant, network_interface_management: NetworkInterfaceManagement) -> Vec<Box<dyn TaskAbsent>> {
    let mut tasks: Vec<Box<dyn TaskAbsent>> = vec![];

    if let NetworkInterfaceManagement::Enabled { network_interface_manager, .. } = &network_interface_management {
        let network_interface_manager = network_interface_manager.clone();
        match parameter {
            ParameterVariant::DeviceInterface(_device_interface) => {
                // TODO: Create task to check if device is present, if not, ignore it
            }
            ParameterVariant::EthernetBridge(ethernet_bridge) => {
                tasks.push(Box::new(tasks::create_ethernet_bridge::CreateEthernetBridge { parameter: ethernet_bridge.value.clone(), network_interface_manager }));
            }
            ParameterVariant::Executor(_executor) => {
                // TODO: Migrate to a task that can handle the executor
            }
            ParameterVariant::GreInterface(gre_interface) => {
                tasks.push(Box::new(tasks::create_gre_interfaces::ManageGreInterface { parameter: gre_interface.value.clone(), network_interface_manager }));
            }
            ParameterVariant::JoinedInterface(joined_interface) => {
                tasks.push(Box::new(tasks::manage_joined_interfaces::ManageJoinedInterface { parameter: joined_interface.value.clone(), network_interface_manager }));
            }
            ParameterVariant::RemotePeerConnectionCheck(_remote_peer_connection_check) => {
                // TODO: Handle remote peer connection checks in its own tasks
            }
        };
    }

    tasks
}

async fn check_task(task: &dyn TaskAbsent, target: ParameterTarget) -> Result<TaskStateFulfilled, TaskExecutionError> {
    match target {
        ParameterTarget::Present => task.check_present()
            .await
            .map_err(|error| TaskExecutionError::DetermineSystemStateBefore { task_name: task.description(), error }),
        ParameterTarget::Absent => task.check_absent()
            .await
            .map_err(|error| TaskExecutionError::DetermineSystemStateBefore { task_name: task.description(), error }),
    }
}

async fn make_task(task: &dyn TaskAbsent, target: ParameterTarget) -> Result<Success, TaskExecutionError> {
    match target {
        ParameterTarget::Present => task.make_present()
            .await
            .map_err(|error| TaskExecutionError::DuringTaskExecution { task_name: task.description(), error }),
        ParameterTarget::Absent => task.make_absent()
            .await
            .map_err(|error| TaskExecutionError::DuringTaskExecution { task_name: task.description(), error }),
    }
}

pub(crate) async fn run_individual_task(task: &dyn TaskAbsent, target: ParameterTarget) -> Result<Outcome, TaskExecutionError> {
    let is_fulfilled = check_task(task, target).await?;
    
    let outcome = match is_fulfilled {
        TaskStateFulfilled::Yes => {
            Outcome::Unchanged
        }
        TaskStateFulfilled::No | TaskStateFulfilled::Unchecked => {
            make_task(task, target)
                .await
                .map(Outcome::Changed)?
        }
    };

    if let Outcome::Changed(_) = outcome {
        let fulfillment = check_task(task, target).await?;
        match fulfillment {
            TaskStateFulfilled::Yes | TaskStateFulfilled::Unchecked => {}, // do nothing
            TaskStateFulfilled::No => {
                return Err(TaskExecutionError::UnfulfilledTask { task_name: task.description() });
            }
        }
    }

    Ok(outcome)
}

pub async fn run_tasks(peer_configuration: PeerConfiguration, network_interface_managent: NetworkInterfaceManagement, metrics_manager: NetworkMetricsManagerRef) -> HashMap<ParameterVariant, Result<Outcome, TaskExecutionError>> {
    let mut resolver = PeerConfigurationDependencyResolver::new(peer_configuration.clone());

    let mut results = HashMap::new();
    while let Some(parameter) = resolver.next_parameter() {
        let target = parameter.target();
        let tasks = resolve_tasks(&parameter, network_interface_managent.clone());
        let mut outcome_for_parameter: Result<Outcome, TaskExecutionError> = Ok(Outcome::Unchanged);
        
        for task in tasks {
            let result = run_individual_task(task.as_ref(), target).await;
            match result {
                Ok(outcome) => {
                    match outcome {
                        Outcome::DryRun | Outcome::Unchanged => {
                            // if the task is unchanged, we continue to the next task
                            continue;
                        }
                        Outcome::Changed(success) => {
                            // if the task changed, we update the outcome for this parameter
                            outcome_for_parameter = Ok(Outcome::Changed(success));
                        }
                        Outcome::Failed => {
                            outcome_for_parameter = Ok(Outcome::Failed);
                            resolver.mark_current_parameter_failed();
                            break; // no need to continue with other tasks for this parameter
                        }
                    }
                }
                Err(error) => {
                    // if the task failed, we update the outcome for this parameter
                    outcome_for_parameter = Err(error);
                    resolver.mark_current_parameter_failed();
                    break; // no need to continue with other tasks for this parameter
                }
            }
        }
        results.insert(parameter, outcome_for_parameter);


    }

    // Now we can run the tasks that are not conformant to the peer configuration dependency resolver
    // TODO: fix
    let mut remote_peer_connection_checks = peer_configuration.remote_peer_connection_checks.clone().into_iter().collect::<Vec<_>>();
    remote_peer_connection_checks.sort_by(|a, b| a.target.cmp(&b.target));

    let remote_peers = remote_peer_connection_checks.into_iter()
        .map(|connection_check| (connection_check.value.remote_peer_id, connection_check.value.remote_ip))
        .collect::<HashMap<_, _>>(); //TODO split into multiple tasks
    let connection_check_task: Box<dyn TaskAbsent> = Box::new(tasks::setup_cluster_metrics::SetupClusterMetrics {
        remote_peers,
        metrics_manager,
    });
    let connection_check_result = run_individual_task(connection_check_task.as_ref(), ParameterTarget::Present).await;
    let connection_check_parameters = peer_configuration.remote_peer_connection_checks.into_iter().collect::<Vec<_>>();
    if let Some(parameter) = connection_check_parameters.first() {
        let variant = ParameterVariant::RemotePeerConnectionCheck(Box::new(parameter.clone()));
        // TODO: This is reflecting the entire result, but uses the first parameter as the key.
        results.insert(variant, connection_check_result);
    }

    results
}