use std::fmt::{Display, Formatter};
use std::iter;

use clap::builder::PossibleValue;
use strum::IntoEnumIterator;

use crate::core::types::Package;

const PACKAGE_SELECTION_ALL: &str = "all";

#[derive(Clone, Debug, Default)]
pub enum PackageSelection {
    #[default]
    All,
    Single(Package),
}
impl PackageSelection {
    pub fn iter(&self) -> Box<dyn Iterator<Item=Package>> {
        match self {
            PackageSelection::Single(package) => Box::new(iter::once(Clone::clone(package))),
            PackageSelection::All => Box::new(Package::iter()),
        }
    }
}
impl Display for PackageSelection {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PackageSelection::Single(package) => write!(f, "{}", package),
            PackageSelection::All => write!(f, "{}", PACKAGE_SELECTION_ALL),
        }
    }
}

impl clap::ValueEnum for PackageSelection {
    fn value_variants<'a>() -> &'a [PackageSelection] {
        let variants = Package::iter()
            .map(PackageSelection::Single)
            .chain(iter::once(PackageSelection::All))
            .collect::<Vec<PackageSelection>>();

        Box::leak(variants.into())
    }
    fn to_possible_value(&self) -> Option<PossibleValue> {
        match self {
            PackageSelection::Single(package) => Some(PossibleValue::new(package.ident())),
            PackageSelection::All => Some(PossibleValue::new(PACKAGE_SELECTION_ALL)),
        }
    }
}
