use std::collections::HashMap;
use crate::common::task::dependency::PeerConfigurationDependencyResolver;
use crate::common::task::{Success, TaskAbsent, TaskStateFulfilled};
use opendut_types::peer::configuration::{ParameterTarget, ParameterVariant, PeerConfiguration};
use crate::common::task::runner::{Outcome, TaskExecutionError};
use crate::common::task::task_resolver::TaskResolver;

pub async fn run_tasks(
    peer_configuration: PeerConfiguration,
    task_resolver: impl TaskResolver,
) -> HashMap<ParameterVariant, Result<Outcome, TaskExecutionError>> {
    let mut resolver = PeerConfigurationDependencyResolver::new(peer_configuration.clone());

    // TODO: change result to a vector, preserve parameter id
    let mut results = HashMap::new();
    while let Some(parameter) = resolver.next_parameter() {
        let target = parameter.target();
        let tasks = task_resolver.resolve_tasks(&parameter);

        let outcome_for_parameter = run_multiple_tasks(&tasks, target, &mut resolver).await;
        results.insert(parameter, outcome_for_parameter);

    }

    for task in task_resolver.additional_tasks() {
        let tasks = task.tasks;
        let target = task.parameter.target();
        let outcome_for_parameter = run_multiple_tasks(&tasks, target, &mut resolver).await;
        results.insert(task.parameter, outcome_for_parameter);
    }

    results
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
    outcome_for_parameter
}
