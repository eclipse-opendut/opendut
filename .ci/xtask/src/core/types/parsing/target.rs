use std::fmt::{Display, Formatter};
use std::iter;

use clap::builder::PossibleValue;

use crate::core::types::Target;

const TARGET_SELECTION_ALL: &str = "all";

fn supported_target_variants() -> Vec<Target> {
    vec![Target::Arm64, Target::Armhf, Target::X86_64]
}

#[derive(Clone, Debug, Default)]
pub enum TargetSelection {
    #[default]
    Default,
    All,
    Single(Target),
}
impl TargetSelection {
    pub fn iter(&self) -> Box<dyn Iterator<Item=Target>> {
        match self {
            TargetSelection::Single(target) => Box::new(
                iter::once(Clone::clone(target))
            ),
            TargetSelection::Default => Box::new(
                iter::once(Target::default())
            ),
            TargetSelection::All => Box::new(
                supported_target_variants().into_iter()
            ),
        }
    }
}
impl Display for TargetSelection {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TargetSelection::Default => write!(f, "{}", Target::default().triple()),
            TargetSelection::Single(target) => write!(f, "{}", target.triple()),
            TargetSelection::All => write!(f, "{}", TARGET_SELECTION_ALL),
        }
    }
}

impl clap::ValueEnum for TargetSelection {
    fn value_variants<'a>() -> &'a [TargetSelection] {
        let variants = supported_target_variants().into_iter()
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
