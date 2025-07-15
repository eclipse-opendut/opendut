use serde::Serialize;
use std::hash::{Hash, Hasher};
use std::time::SystemTime;
use uuid::Uuid;

mod value;
pub use value::ParameterValue;


#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Parameter<V: ParameterValue> {
    pub id: ParameterId,
    pub dependencies: Vec<ParameterId>,
    pub target: ParameterTarget,
    pub value: V,
}

impl<V: ParameterValue> Hash for Parameter<V> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
pub struct ParameterId(pub Uuid);

#[derive(Clone, Copy, Debug, Ord, PartialOrd, PartialEq, Eq, Serialize)]
pub enum ParameterTarget {
    Absent,
    Present,
}

#[derive(Debug, Clone)]
pub struct PeerConfigurationState {
    pub parameter_states: Vec<PeerConfigurationParameterState>
}

#[derive(Debug, Clone)]
pub struct PeerConfigurationParameterState {
    pub id: ParameterId,
    pub timestamp: SystemTime,
    pub state: ParameterTargetState,
}

#[derive(Debug, Clone)]
pub enum ParameterTargetState {
    Absent,
    Present,
    WaitingForDependencies(Vec<ParameterId>),
    Error(ParameterTargetStateError),
}

#[derive(Debug, Clone)]
pub enum ParameterTargetStateError {
    CreatingFailed(ParameterTargetStateErrorCreatingFailed),
    RemovingFailed(ParameterTargetStateErrorRemovingFailed),
}

#[derive(Debug, Clone)]
pub enum ParameterTargetStateErrorCreatingFailed {
    UnclassifiedError(String),
}

#[derive(Debug, Clone)]
pub enum ParameterTargetStateErrorRemovingFailed {
    UnclassifiedError(String),
}

#[cfg(test)]
mod tests {
    use crate::peer::configuration::ParameterTarget;

    #[test]
    fn test_sort_parameter_target() {
        let mut example_set = [ParameterTarget::Present, ParameterTarget::Absent, ParameterTarget::Present];
        example_set.sort();
        assert!(example_set.starts_with(&[ParameterTarget::Absent]));        
        assert!(example_set.ends_with(&[ParameterTarget::Present]));        
    }
}
