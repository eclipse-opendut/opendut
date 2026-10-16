pub mod create;
pub mod delete;
pub mod describe;
pub mod list;

use opendut_model::viper::{ViperRunDeployment, ViperRunId, ViperTestId};
use serde::Serialize;

#[derive(Serialize, Debug)]
struct SerializableViperRunDeployment {
    run_id: ViperRunId,
    test_id: ViperTestId,
}

impl From<ViperRunDeployment> for SerializableViperRunDeployment {
    fn from(deployment: ViperRunDeployment) -> Self {
        let ViperRunDeployment { run_id, test_id } = deployment;

        Self { run_id, test_id }
    }
}
