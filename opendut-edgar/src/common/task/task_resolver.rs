use opendut_model::peer::configuration::ParameterVariant;
use crate::common::task::TaskAbsent;

pub trait TaskResolver {
    /// Resolves tasks based on the provided parameter variant.
    fn resolve_tasks(&self, parameter: &ParameterVariant) -> Vec<Box<dyn TaskAbsent>>;
    /// Additional tasks that are not conformant to the peer configuration dependency resolver
    /// see also opendut-model/proto/opendut/model/peer/configuration/api.proto feedback to CARL via ParameterId -> present/absent
    fn additional_tasks(&self) -> Vec<AdditionalTasks>;
}

pub struct AdditionalTasks {
    pub tasks: Vec<Box<dyn TaskAbsent>>,
    pub parameter: ParameterVariant,
}
