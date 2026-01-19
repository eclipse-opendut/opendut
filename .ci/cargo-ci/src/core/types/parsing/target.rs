use std::fmt::{Display, Formatter};
use std::iter;

use clap::builder::PossibleValue;
use strum::IntoEnumIterator;

use crate::Arch;


#[derive(Clone, Debug, Default)]
pub enum TargetSelection {
    #[default]
    Default,
    Single(Arch),
}
impl TargetSelection {
    pub fn iter(&self) -> Box<dyn Iterator<Item=Arch>> {
        match self {
            TargetSelection::Single(target) => Box::new(
                iter::once(Clone::clone(target))
            ),
            TargetSelection::Default => Box::new(
                iter::once(Arch::default())
            ),
        }
    }
}
impl Display for TargetSelection {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TargetSelection::Default => write!(f, "{}", Arch::default()),
            TargetSelection::Single(target) => write!(f, "{target}"),
        }
    }
}

impl clap::ValueEnum for TargetSelection {
    fn value_variants<'a>() -> &'a [TargetSelection] {
        let variants = Arch::iter()
            .map(TargetSelection::Single)
            .collect::<Vec<TargetSelection>>();

        Box::leak(variants.into())
    }
    fn to_possible_value(&self) -> Option<PossibleValue> {
        match self {
            TargetSelection::Default => None,
            TargetSelection::Single(target) => Some(PossibleValue::new(target.triple())),
        }
    }
}
