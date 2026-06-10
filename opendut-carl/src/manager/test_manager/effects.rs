use tracing::error;
use opendut_model::viper::ViperRunDeployment;
use crate::resource::{manager::{ResourceManagerRef, SubscriptionEvent}};

pub(crate) async fn register(resource_manager: ResourceManagerRef) {
    schedule_fetch_source_code_when_test_run_deployment_available(resource_manager).await;
}

#[tracing::instrument(skip_all)]
async fn schedule_fetch_source_code_when_test_run_deployment_available(resource_manager: ResourceManagerRef) { //TODO also await peer online
    resource_manager.spawn_event_listener::<ViperRunDeployment>(
        {
            let resource_manager = resource_manager.clone();
            async move |event| {
                if let SubscriptionEvent::Inserted { id: run_id, value: viper_run_deployment } = event {
                    let test_id = viper_run_deployment.test_id;

                    let result = resource_manager.resources_mut(async |resources| {
                        resources.fetch_source_code(test_id).await
                    }).await
                        .expect("Persistence Error while fetching source code."); //Todo: error handling

                    if let Err(error) = result {
                        error!("Error while fetching source code for run <{run_id}: \n{error}");
                        //TODO handle result in some way (retry or decide not to send updated peer configuration)
                    }
                }
            }
        }
    ).await;
}
