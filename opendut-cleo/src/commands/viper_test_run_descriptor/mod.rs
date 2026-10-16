pub mod create;
pub mod delete;
pub mod describe;
pub mod list;

use opendut_model::peer::PeerId;
use opendut_model::viper::{ViperSourceId, ViperTestId, ViperTestName, ViperTestParameters, ViperTestRunDescriptor};
use serde::Serialize;

#[derive(Serialize, Debug)]
struct SerializableViperTestRun {
    name: ViperTestName,
    id: ViperTestId,
    source: ViperSourceId,
    peer: PeerId,
    parameters: ViperTestParameters,
}

impl From<ViperTestRunDescriptor> for SerializableViperTestRun {
    fn from(test: ViperTestRunDescriptor) -> Self {
        let ViperTestRunDescriptor { name, id, source, peer, parameters } = test;

        Self { name, id, source, peer, parameters }
    }
}
