use std::collections::HashSet;
use anyhow::Context;
use tracing::error;
use opendut_model::peer::configuration::{parameter, ParameterTarget, PeerConfiguration};
use opendut_model::viper::{TestRunSourceCode, ViperRunDeployment, ViperRunId, ViperTestId};
use opendut_viper_rt::compile::SourceCode;
use opendut_viper_rt::source::Source;
use opendut_viper_rt::ViperRuntime;
use crate::manager::viper_manager::fetch_source_code::{fetch_source_code, FetchError};
use crate::resource::{manager::{ResourceManagerRef, SubscriptionEvent}};
use crate::resource::manager::ResourcesStorageApi;

pub(crate) async fn register(resource_manager: ResourceManagerRef) {
    let fetch_source_code_closure = async move |viper_runtime: ViperRuntime, source: &Source| {
        let Source { identifier: test_suite_identifier, location, .. } = source.clone();

        let source_code = viper_runtime.fetch_source_code(source).await
            .map_err(|error| FetchError::Compilation { test_suite_identifier, location, cause: error })?;
        Ok(source_code)
    };

    schedule_fetch_source_code_when_test_run_deployment_available(resource_manager, fetch_source_code_closure).await;
}

#[tracing::instrument(skip_all)]
async fn schedule_fetch_source_code_when_test_run_deployment_available(
    resource_manager: ResourceManagerRef,
    fetch_source_code_closure: impl (AsyncFnOnce(ViperRuntime, &Source) -> Result<SourceCode, FetchError>) + Send + Sync + Clone + 'static
) {
    //TODO also await peer online
    resource_manager.spawn_event_listener::<ViperRunDeployment>({
        let resource_manager = resource_manager.clone();

        async move |event| {
            let resource_manager = resource_manager.clone();
            let fetch_source_code_closure = fetch_source_code_closure.clone();

            if let SubscriptionEvent::Inserted { id: run_id, value: viper_run_deployment } = event {
                let test_id = viper_run_deployment.test_id;

                let result = fetch_source_code(resource_manager.clone(), test_id, fetch_source_code_closure).await;

                match result {
                    Ok(source_code) => {
                        let result = update_peer_configuration(resource_manager, run_id, test_id, source_code).await;
                        if let Err(error) = result {
                            error!("Error while fetching peer configuration for test <{test_id}>: \n{error}")
                        }
                    },
                    Err(error) => {
                        //TODO handle result in some way (retry or decide not to send updated peer configuration)
                        error!("Error while fetching source code for run <{run_id}>: \n{error}")
                    },
                }
            }
        }
    }).await;
}

async fn update_peer_configuration(
    resource_manager: ResourceManagerRef,
    run_id: ViperRunId,
    test_id: ViperTestId,
    source_code: SourceCode
) -> anyhow::Result<()> {

    resource_manager.resources_mut(async |resources| {
        let peer_id= resources.get_peer_id_for_test(test_id)?;

        let mut peer_configuration = resources.get::<PeerConfiguration>(peer_id)?
            .unwrap_or_default();

        let test_run_report = parameter::TestRunReport {
            run_id,
            source_code: TestRunSourceCode { inner: source_code },
        };

        peer_configuration.test_run_reports.set(test_run_report, ParameterTarget::Present, HashSet::new()); //Todo: Clean up completed test runs

        resources.insert(peer_id, peer_configuration)
            .context("Error while inserting peer configuration while updating test run report parameter.")?;

        anyhow::Ok(())
    }).await??;

    Ok(())
}

#[cfg(test)]
mod test {
    use std::time::Duration;
    use tokio::sync::mpsc;
    use crate::manager::testing::{SourceCodeFixture, ViperRunDeploymentFixture};
    use crate::resource::manager::ResourceManager;
    use super::*;

    #[tokio::test]
    async fn should_trigger_fetching_source_code_when_test_run_deployment_available() -> anyhow::Result<()> {
        let (sender, mut receiver) = mpsc::channel(1);
        let (resource_manager, _resource_manager_cancel) = ResourceManager::new_in_memory();

        let source_code = SourceCodeFixture::new().source_code;
        let simulate_fetch_source_code = {
            let source_code = Clone::clone(&source_code);
            async move |_viper_runtime: ViperRuntime, _source: &Source| -> Result<SourceCode, FetchError> {
                sender.send(()).await.unwrap();

                Ok(source_code)
            }
        };

        schedule_fetch_source_code_when_test_run_deployment_available(
            resource_manager.clone(),
            simulate_fetch_source_code
        ).await;

        let viper_run_deployment = ViperRunDeploymentFixture::create(resource_manager.clone()).await?;

        let run_id = viper_run_deployment.id;
        let peer_id = viper_run_deployment.test.cluster.descriptor.leader;

        let fetch_was_triggered = tokio::time::timeout(
            Duration::from_secs(5),
            receiver.recv(),
        ).await;
        assert_eq!(fetch_was_triggered, Ok(Some(())));

        let peer_configuration = resource_manager.get::<PeerConfiguration>(peer_id).await?;

        let test_run_report = peer_configuration.unwrap()
            .test_run_reports.values
            .values()
            .next()
            .map(|value| Clone::clone(&value.value));

        let expected_test_run_report = Some(parameter::TestRunReport { run_id, source_code: TestRunSourceCode { inner: source_code }});
        assert_eq!(test_run_report, expected_test_run_report);

        Ok(())
    }
}
