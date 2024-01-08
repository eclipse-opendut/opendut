use clap::builder::PossibleValue;
use clap::ValueEnum;
use strum::IntoEnumIterator;

use crate::core::types::arch::Arch;

/// Architecture which is actively supported by openDuT
#[derive(Clone, Copy, Debug, strum::EnumIter)]
pub enum Target {
    X86_64,
    Armhf,
    Arm64,
}

impl Target {
    pub fn arch(&self) -> Arch {
        match self {
            Target::X86_64 => Arch::X86_64,
            Target::Armhf => Arch::Armhf,
            Target::Arm64 => Arch::Arm64,
        }
    }
    pub fn triple(&self) -> String {
        self.arch().triple()
    }
}

impl Default for Target {
    fn default() -> Self {
        let arch_triple = crate::build::BUILD_TARGET;
        let ignore_case = true;
        Target::from_str(arch_triple, ignore_case).unwrap()
    }
}


impl clap::ValueEnum for Target {
    fn value_variants<'a>() -> &'a [Target] {
        Box::leak(Self::iter().collect::<Vec<Target>>().into())
    }
    fn to_possible_value(&self) -> Option<PossibleValue> {
        Some(PossibleValue::new(self.arch().triple()))
    }
}
