use std::fmt::{Display, Formatter};
use std::iter;

use clap::builder::PossibleValue;
use strum::IntoEnumIterator;

use crate::core::types::Arch;

const ARCH_SELECTION_ALL: &str = "all";

#[derive(Clone, Debug, Default)]
pub enum ArchSelection {
    #[default]
    Default,
    All,
    Single(Arch),
}
impl ArchSelection {
    pub fn iter(&self) -> Box<dyn Iterator<Item=Arch>> {
        match self {
            ArchSelection::Single(arch) => Box::new(
                iter::once(Clone::clone(arch))
            ),
            ArchSelection::Default => Box::new(
                iter::once(Arch::default())
            ),
            ArchSelection::All => Box::new(
                Arch::iter()
            ),
        }
    }
}
impl Display for ArchSelection {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ArchSelection::Default => write!(f, "{}", Arch::default()),
            ArchSelection::Single(arch) => write!(f, "{}", arch),
            ArchSelection::All => write!(f, "{}", ARCH_SELECTION_ALL),
        }
    }
}

impl clap::ValueEnum for ArchSelection {
    fn value_variants<'a>() -> &'a [ArchSelection] {
        let variants = Arch::iter()
            .map(ArchSelection::Single)
            .chain(iter::once(ArchSelection::All))
            .collect::<Vec<ArchSelection>>();

        Box::leak(variants.into())
    }
    fn to_possible_value(&self) -> Option<PossibleValue> {
        match self {
            ArchSelection::Default => None,
            ArchSelection::Single(arch) => Some(PossibleValue::new(arch.triple())),
            ArchSelection::All => Some(PossibleValue::new(ARCH_SELECTION_ALL)),
        }
    }
}
