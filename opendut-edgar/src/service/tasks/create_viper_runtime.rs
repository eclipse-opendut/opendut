use async_trait::async_trait;
use opendut_model::peer::configuration::parameter;
use crate::common::task::{Success, Task, TaskAbsent, TaskStateFulfilled};
use crate::service::viper_run_manager::ViperRunManagerRef;

pub struct CreateViperTestRun {
    pub parameter: parameter::TestRunReport,
    pub viper_run_manager: ViperRunManagerRef,
}

#[async_trait]
impl Task for CreateViperTestRun {
    fn description(&self) -> String {
        format!("Create VIPER test run '{}'", self.parameter.run_id)
    }

    async fn check_present(&self) -> anyhow::Result<TaskStateFulfilled> {
        if self.viper_run_manager.contains_test_run(&self.parameter.run_id).await {
            Ok(TaskStateFulfilled::Yes)
        } else {
            Ok(TaskStateFulfilled::No)
        }
    }

    async fn make_present(&self) -> anyhow::Result<Success> {
        let run_id = self.parameter.run_id;
        let source_code = Clone::clone(&self.parameter.source_code);
        let parameters = Clone::clone(&self.parameter.parameters);

        self.viper_run_manager.start_test_run(run_id, source_code, parameters).await;

        Ok(Success::default())
    }
}

#[async_trait]
impl TaskAbsent for CreateViperTestRun {
    async fn check_absent(
        &self,
    ) -> anyhow::Result<TaskStateFulfilled> {
        if self.viper_run_manager.contains_test_run(&self.parameter.run_id).await {
            Ok(TaskStateFulfilled::No)
        } else {
            Ok(TaskStateFulfilled::Yes)
        }
    }

    async fn make_absent(&self) -> anyhow::Result<Success> {
        self.viper_run_manager.abort_test_run(
            &self.parameter.run_id,
        ).await;

        Ok(Success::default())
    }
}
