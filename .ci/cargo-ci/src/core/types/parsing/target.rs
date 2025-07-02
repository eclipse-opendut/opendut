use std::fmt::{Display, Formatter};
use std::iter;

use clap::builder::PossibleValue;
use strum::IntoEnumIterator;

use crate::Arch;

const TARGET_SELECTION_ALL: &str = "all";

#[derive(Clone, Debug, Default)]
pub enum TargetSelection {
    #[default]
    Default,
    All,
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
            TargetSelection::All => Box::new(
                Arch::iter()
            ),
        }
    }
}
impl Display for TargetSelection {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TargetSelection::Default => write!(f, "{}", Arch::default()),
            TargetSelection::Single(target) => write!(f, "{target}"),
            TargetSelection::All => write!(f, "{TARGET_SELECTION_ALL}"),
        }
    }
}

impl clap::ValueEnum for TargetSelection {
    fn value_variants<'a>() -> &'a [TargetSelection] {
        let variants = Arch::iter()
            .map(TargetSelection::Single)
            .chain(iter::once(TargetSelection::All))
            .collect::<Vec<TargetSelection>>();

        Box::leak(variants.into())
    }
    fn to_possible_value(&self) -> Option<PossibleValue> {
        match self {
            TargetSelection::Default => None,
            TargetSelection::Single(target) => Some(PossibleValue::new(target.triple())),
            TargetSelection::All => Some(PossibleValue::new(TARGET_SELECTION_ALL)),
        }
    }
}
