use serde::Serialize;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::time::SystemTime;
use uuid::Uuid;

mod value;
pub use value::ParameterValue;
use crate::OPENDUT_UUID_NAMESPACE;

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

impl ParameterId {
    pub fn from_hashable<T: Hash>(value: &T) -> Self {
        let mut hasher = DefaultHasher::new(); //ID not stable across Rust releases
        value.hash(&mut hasher);
        let id = hasher.finish();
        let id = Uuid::new_v5(&OPENDUT_UUID_NAMESPACE, &id.to_le_bytes());
        ParameterId(id)
    }
}

#[derive(Clone, Copy, Debug, Ord, PartialOrd, PartialEq, Eq, Serialize)]
pub enum ParameterTarget {
    Absent,
    Present,
}

#[derive(Debug, Clone)]
pub struct PeerConfigurationState {
    pub parameter_states: Vec<PeerConfigurationParameterState>
}

impl PeerConfigurationState {
    pub fn is_ready(&self) -> bool {
        self.parameter_states.iter().all(PeerConfigurationParameterState::is_ready)
    }
}


#[derive(Debug, Clone)]
pub struct PeerConfigurationParameterState {
    pub id: ParameterId,
    pub timestamp: SystemTime,
    pub detected_state: ParameterDetectedStateKind,
}

impl PeerConfigurationParameterState {
    pub fn is_ready(&self) -> bool {
        matches!(self.detected_state, ParameterDetectedStateKind::Present | ParameterDetectedStateKind::Absent)
    }
}

#[derive(Debug, Clone)]
pub enum ParameterDetectedStateKind {
    Present,
    Absent,
    Creating,
    Removing,
    Error(ParameterDetectedStateError),
}


#[derive(Debug, Clone)]
pub struct EdgePeerConfigurationState {
    pub parameter_states: Vec<EdgePeerConfigurationParameterState>
}

/// State of a parameter on the edge peer side.
#[derive(Debug, Clone)]
pub struct EdgePeerConfigurationParameterState {
    pub id: ParameterId,
    pub timestamp: SystemTime,
    pub detected_state: ParameterEdgeDetectedStateKind,
}

#[derive(Debug, Clone)]
pub enum ParameterEdgeDetectedStateKind {
    Present,
    Absent,
    Error(ParameterDetectedStateError),
}


#[derive(Debug, Clone)]
pub struct ParameterDetectedStateError {
    pub kind: ParameterDetectedStateErrorKind,
    pub cause: ParameterDetectedStateErrorCause,
}

#[derive(Debug, Clone)]
pub enum ParameterDetectedStateErrorCause {
    Unclassified(String),
    MissingDependencies(Vec<ParameterId>),
}

#[derive(Debug, Clone)]
pub enum ParameterDetectedStateErrorKind {
    CreatingFailed,
    RemovingFailed,
    CheckPresentFailed,
    CheckAbsentFailed,
    WaitingForDependenciesFailed,
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
