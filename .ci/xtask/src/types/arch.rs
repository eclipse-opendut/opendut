use std::fmt::{Display, Formatter};

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
}

impl Display for Arch {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.triple())
    }
}
