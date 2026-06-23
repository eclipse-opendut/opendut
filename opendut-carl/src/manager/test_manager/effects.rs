use tracing::error;
use opendut_model::viper::ViperRunDeployment;
use opendut_viper_rt::compile::SourceCode;
use opendut_viper_rt::source::Source;
use opendut_viper_rt::ViperRuntime;
use crate::manager::test_manager::fetch_source_code::FetchError;
use crate::resource::{manager::{ResourceManagerRef, SubscriptionEvent}};

pub(crate) async fn register(resource_manager: ResourceManagerRef) {
    let fetch_source_code_closure = async move |viper_runtime: ViperRuntime, source: &Source| {
        let source_code = viper_runtime.fetch_source_code(source).await?;
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

                // Todo: Don't block resource manager throughout fetching (resources_mut)
                let result = resource_manager.resources_mut(async |resources| {
                    resources.fetch_source_code(test_id, fetch_source_code_closure).await
                }).await
                    .expect("Persistence Error while fetching source code."); //Todo: error handling

                if let Err(error) = result {
                    error!("Error while fetching source code for run <{run_id}: \n{error}");
                    //TODO handle result in some way (retry or decide not to send updated peer configuration)
                }
            }
        }
    }).await;
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

        let simulate_fetch_source_code = async move |_viper_runtime: ViperRuntime, _source: &Source| -> Result<SourceCode, FetchError> {
            sender.send(()).await.unwrap();

            let source_code = SourceCodeFixture::new().source_code;
            Ok(source_code)
        };

        schedule_fetch_source_code_when_test_run_deployment_available(
            resource_manager.clone(),
            simulate_fetch_source_code
        ).await;

        ViperRunDeploymentFixture::create(resource_manager.clone()).await?;

        let fetch_was_triggered = tokio::time::timeout(
            Duration::from_secs(5),
            receiver.recv(),
        ).await;

        assert_eq!(fetch_was_triggered, Ok(Some(())));
        Ok(())
    }
}
