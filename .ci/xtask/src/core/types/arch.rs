use std::fmt::{Display, Formatter};

use clap::builder::PossibleValue;
use strum::IntoEnumIterator;


#[derive(Clone, Debug, strum::EnumIter)]
pub enum Arch {
    X86_64,
    Armhf,
    Arm64,
}

impl Arch {
    pub fn triple(&self) -> String {
        match self {
            Arch::X86_64 => "x86_64-unknown-linux-gnu",
            Arch::Armhf => "armv7-unknown-linux-gnueabihf",
            Arch::Arm64 => "aarch64-unknown-linux-gnu",
        }.to_string()
    }

    pub fn get_or_default(target: Option<Arch>) -> Arch {
        use clap::ValueEnum;

        target.unwrap_or_else(|| {
            let arch_triple = crate::build::BUILD_TARGET;
            log::info!("No target specified. Using default target of machine: {arch_triple}");
            let ignore_case = true;
            Arch::from_str(arch_triple, ignore_case).unwrap()
        })
    }
}

impl Display for Arch {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.triple())
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
