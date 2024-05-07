use std::slice::Iter;
use serde::{Deserialize, Serialize};
use crate::util::CleoArch::{Arm64, Armhf, X86_64, Development};

pub const CLEO_IDENTIFIER: &str = "opendut-cleo";

#[derive(Serialize, Deserialize)]
pub enum CleoArch {
    #[serde(rename="x86_64-unknown-linux-gnu")]
    X86_64,
    #[serde(rename="armv7-unknown-linux-gnueabihf")]
    Armhf,
    #[serde(rename="aarch64-unknown-linux-gnu")]
    Arm64,
    #[serde(rename="development")]
    Development,
}
impl CleoArch {
    pub fn name(&self) -> String {
        match self {
            X86_64 => "opendut-cleo-x86_64-unknown-linux-gnu",
            Armhf => "opendut-cleo-armv7-unknown-linux-gnueabihf",
            Arm64 => "opendut-cleo-aarch64-unknown-linux-gnu",
            Development => "opendut-cleo",
        }.to_string()
    }

    pub fn arch_iterator() -> Iter<'static, CleoArch> {
        static CLEO_ARCH: [CleoArch; 3] = [X86_64, Armhf, Arm64];
        CLEO_ARCH.iter()
    }
}