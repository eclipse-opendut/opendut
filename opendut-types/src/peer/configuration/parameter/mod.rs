use uuid::Uuid;

mod value;
pub use value::ParameterValue;


#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Parameter<V: ParameterValue> {
    pub id: ParameterId,
    pub dependencies: Vec<ParameterId>,
    pub target: ParameterTarget,
    pub value: V,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ParameterId(pub Uuid);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParameterTarget {
    Present,
    Absent,
}

