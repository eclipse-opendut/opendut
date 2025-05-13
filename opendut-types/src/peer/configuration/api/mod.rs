use serde::Serialize;
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
pub struct ParameterId(pub Uuid);

#[derive(Clone, Copy, Debug, Ord, PartialOrd, PartialEq, Eq, Serialize)]
pub enum ParameterTarget {
    Absent,
    Present,
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