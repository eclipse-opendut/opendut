use std::fmt::Formatter;
use clap::builder::PossibleValue;
use clap::ValueEnum;
use strum::IntoEnumIterator;

/// General architecture used somewhere in the build process
#[derive(Clone, Copy, PartialEq, Debug, strum::EnumIter)]
pub enum Arch {
    X86_64,
    Armhf,
    Arm64,
    Wasm,
}
impl Arch {
    pub fn triple(&self) -> String {
        match self {
            Arch::X86_64 => "x86_64-unknown-linux-gnu",
            Arch::Armhf => "armv7-unknown-linux-gnueabihf",
            Arch::Arm64 => "aarch64-unknown-linux-gnu",
            Arch::Wasm => "wasm32-unknown-unknown",
        }.to_string()
    }
}

impl std::fmt::Display for Arch {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.triple())
    }
}


impl Default for Arch {
    fn default() -> Self {
        let arch_triple = crate::build::BUILD_TARGET;
        let ignore_case = true;
        Arch::from_str(arch_triple, ignore_case).unwrap()
    }
}

impl clap::ValueEnum for Arch {
    fn value_variants<'a>() -> &'a [Arch] {
        Box::leak(Self::iter().collect::<Vec<Arch>>().into())
    }
    fn to_possible_value(&self) -> Option<PossibleValue> {
        Some(PossibleValue::new(self.triple()))
    }
}
