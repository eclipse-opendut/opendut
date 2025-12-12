//! This module provides functionality to run tasks based on a peer configuration.
//!
//! Service: PeerConfiguration -> ParameterVariant (variant, id, target) -> [ List of TaskAbsent ] -> Outcome
//! Setup: [ List of Task ] (only present used atm) -> Outcome
//!
//! PeerConfigurationDependencyResolver: PeerConfiguration -> ParameterVariant
//! TaskResolver: ParameterVariant -> [ List of TaskAbsent ]

use std::collections::HashSet;
use std::time::SystemTime;
use crate::common::task::dependency::{ParameterVariantWithDependencies, PeerConfigurationDependencyResolver};
use crate::common::task::runner::{TaskExecutionError};
use crate::common::task::task_resolver::TaskResolver;
use crate::common::task::{Success, TaskAbsent, TaskStateFulfilled};
use opendut_model::peer::configuration::{ParameterId, ParameterTarget, ParameterVariant, PeerConfiguration, EdgePeerConfigurationState, ParameterDetectedStateError, ParameterDetectedStateErrorKind, ParameterDetectedStateErrorCause, EdgePeerConfigurationParameterState, ParameterEdgeDetectedStateKind};

#[derive(Debug)]
pub enum Outcome {
    Changed(Success),
    Unchanged,
}

pub struct CollectedResult {
    pub(crate) items: Vec<ResultItem>,
    pub success: bool,
    unfulfilled_parameters: Vec<ParameterVariantWithDependencies>
}

impl From<CollectedResult> for EdgePeerConfigurationState {
    fn from(value: CollectedResult) -> Self {
        let successful_parameter_ids = value.items.iter().flat_map(|item| {
            if item.outcome.is_ok() {
                Some(item.id)
            } else {
                None
            }
        }).collect::<HashSet<_>>();

        fn make_error(kind: ParameterDetectedStateErrorKind, cause: impl ToString) -> ParameterEdgeDetectedStateKind {
            ParameterEdgeDetectedStateKind::Error(
                ParameterDetectedStateError {
                    kind,
                    cause: ParameterDetectedStateErrorCause::Unclassified(cause.to_string()),
                }
            )
        }

        let mut parameter_states: Vec<EdgePeerConfigurationParameterState> = value.items.into_iter().map(|item| {
            let target = item.parameter.target();
            let state = match item.outcome {
                Ok(outcome) => {
                    match outcome {
                        Outcome::Unchanged | Outcome::Changed(_) => {
                            match target {
                                ParameterTarget::Present => ParameterEdgeDetectedStateKind::Present,
                                ParameterTarget::Absent => ParameterEdgeDetectedStateKind::Absent,
                            }
                        }
                    }
                }
                Err(error) => {
                    match error {
                        TaskExecutionError::DetermineSystemStateBefore { task_name, error } | TaskExecutionError::DetermineSystemStateAfter { task_name, error } => {
                            match target {
                                ParameterTarget::Present => make_error(ParameterDetectedStateErrorKind::CheckPresentFailed, format!("Task '{task_name}' failed. Error {error}")),
                                ParameterTarget::Absent => make_error(ParameterDetectedStateErrorKind::CheckAbsentFailed, format!("Task '{task_name}' failed. Error {error}")),
                            }
                        }
                        TaskExecutionError::DuringTaskExecution { task_name, error } => {
                            match target {
                                ParameterTarget::Present => make_error(ParameterDetectedStateErrorKind::CreatingFailed, format!("Task '{task_name}' failed. Error {error}")),
                                ParameterTarget::Absent => make_error(ParameterDetectedStateErrorKind::RemovingFailed, format!("Task '{task_name}' failed. Error {error}")),
                            }
                        }
                        TaskExecutionError::UnfulfilledTask { task_name } => {
                            let msg = format!("Task '{}' could not fulfill the parameter.", task_name);
                            match target {
                                ParameterTarget::Present => make_error(ParameterDetectedStateErrorKind::CreatingFailed, msg),
                                ParameterTarget::Absent => make_error(ParameterDetectedStateErrorKind::RemovingFailed, msg),
                            }
                        }
                    }
                }
            };

            EdgePeerConfigurationParameterState {
                id: item.id,
                timestamp: item.timestamp,
                detected_state: state,
            }
            
        }).collect();

        let parameters_without_results = value.unfulfilled_parameters.into_iter().map(|parameter| {
            let missing_dependencies = parameter.dependencies.difference(&successful_parameter_ids).cloned().collect::<Vec<_>>();
            let state = ParameterEdgeDetectedStateKind::Error(
                ParameterDetectedStateError {
                    kind: ParameterDetectedStateErrorKind::WaitingForDependenciesFailed,
                    cause: ParameterDetectedStateErrorCause::MissingDependencies(missing_dependencies)
                }
            );

            EdgePeerConfigurationParameterState {
                id: parameter.id,
                timestamp: SystemTime::now(),
                detected_state: state,
            }
        });
        parameter_states.extend(parameters_without_results);

        EdgePeerConfigurationState {
            parameter_states,
        }
    }
}

#[allow(unused)]
#[derive(Debug)]
pub struct ResultItem {
    pub id: ParameterId,
    pub parameter: ParameterVariant,
    pub outcome: Result<Outcome, TaskExecutionError>,
    pub timestamp: SystemTime,
}


pub async fn run_tasks(
    peer_configuration: PeerConfiguration,
    task_resolver: impl TaskResolver,
) -> CollectedResult {
    let mut resolver = PeerConfigurationDependencyResolver::new(peer_configuration.clone());

    let mut results = CollectedResult { items: vec![], success: false, unfulfilled_parameters: vec![] };
    while let Some(parameter) = resolver.next_parameter() {
        let target = parameter.target();
        let tasks = task_resolver.resolve_tasks(&parameter);

        let outcome_for_parameter = run_multiple_tasks(&tasks, target, &mut resolver).await;
        results.items.push(ResultItem { id: parameter.id(), parameter, outcome: outcome_for_parameter, timestamp: SystemTime::now() });
    }

    for task in task_resolver.additional_tasks() {
        let tasks = task.tasks;
        let target = task.parameter.target();
        let outcome_for_parameter = run_multiple_tasks(&tasks, target, &mut resolver).await;
        results.items.push(ResultItem { id: task.parameter.id(), parameter: task.parameter, outcome: outcome_for_parameter, timestamp: SystemTime::now() });
    }

    results.success = resolver.success();
    results.unfulfilled_parameters = resolver.unfulfilled();
    results
}

async fn check_task(task: &dyn TaskAbsent, target: ParameterTarget) -> anyhow::Result<TaskStateFulfilled> {
    match target {
        ParameterTarget::Present => task.check_present().await,
        ParameterTarget::Absent => task.check_absent().await,
    }
}

async fn make_task(task: &dyn TaskAbsent, target: ParameterTarget) -> anyhow::Result<Success> {
    match target {
        ParameterTarget::Present => task.make_present().await,
        ParameterTarget::Absent => task.make_absent().await,
    }
}

pub(crate) async fn run_individual_task(task: &dyn TaskAbsent, target: ParameterTarget) -> Result<Outcome, TaskExecutionError> {
    let is_fulfilled = check_task(task, target).await
        .map_err(|error| TaskExecutionError::DetermineSystemStateBefore { task_name: task.description(), error })?;

    let outcome = match is_fulfilled {
        TaskStateFulfilled::Yes => {
            Outcome::Unchanged
        }
        TaskStateFulfilled::No | TaskStateFulfilled::Unchecked => {
            make_task(task, target)
                .await
                .map_err(|error| TaskExecutionError::DuringTaskExecution { task_name: task.description(), error })
                .map(Outcome::Changed)?
        }
    };

    if let Outcome::Changed(_) = outcome {
        let fulfillment = check_task(task, target).await
            .map_err(|error| TaskExecutionError::DetermineSystemStateAfter { task_name: task.description(), error })?;
        match fulfillment {
            TaskStateFulfilled::Yes | TaskStateFulfilled::Unchecked => {}, // do nothing
            TaskStateFulfilled::No => {
                return Err(TaskExecutionError::UnfulfilledTask { task_name: task.description() });
            }
        }
    }

    Ok(outcome)
}

async fn run_multiple_tasks(
    tasks: &[Box<dyn TaskAbsent>],
    target: ParameterTarget,
    resolver: &mut PeerConfigurationDependencyResolver,
) -> Result<Outcome, TaskExecutionError> {
    let mut outcome_for_parameter: Result<Outcome, TaskExecutionError> = Ok(Outcome::Unchanged);

    for task in tasks {
        let result = run_individual_task(task.as_ref(), target).await;
        match result {
            Ok(outcome) => {
                match outcome {
                    Outcome::Unchanged => {
                        // if the task is unchanged, we continue to the next task
                        continue;
                    }
                    Outcome::Changed(success) => {
                        // if the task changed, we update the outcome for this parameter
                        outcome_for_parameter = Ok(Outcome::Changed(success));
                    }
                }
            }
            Err(error) => {
                // if the task failed, we update the outcome for this parameter
                outcome_for_parameter = Err(error);
                resolver.mark_current_parameter_as_failed();
                break; // no need to continue with other tasks for this parameter
            }
        }
    }
    outcome_for_parameter
}
