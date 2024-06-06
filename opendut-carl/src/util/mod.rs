use std::slice::Iter;
use serde::{Deserialize, Serialize};


pub const CLEO_IDENTIFIER: &str = "opendut-cleo";
pub const EDGAR_IDENTIFIER: &str = "opendut-edgar";

#[derive(Serialize, Deserialize)]
pub enum CleoArch {
    #[serde(rename="x86_64-unknown-linux-gnu")]
    X86_64,
    #[serde(rename="armv7-unknown-linux-gnueabihf")]
    Armhf,
    #[serde(rename="aarch64-unknown-linux-gnu")]
    Arm64,
}
impl CleoArch {
    pub fn distribution_name(&self) -> String {
        match self {
            CleoArch::X86_64 => "opendut-cleo-x86_64-unknown-linux-gnu",
            CleoArch::Armhf => "opendut-cleo-armv7-unknown-linux-gnueabihf",
            CleoArch::Arm64 => "opendut-cleo-aarch64-unknown-linux-gnu",
        }.to_string()
    }

    pub fn arch_iterator() -> Iter<'static, CleoArch> {
        static CLEO_ARCH: [CleoArch; 3] = [CleoArch::X86_64, CleoArch::Armhf, CleoArch::Arm64];
        CLEO_ARCH.iter()
    }
}

#[derive(Serialize, Deserialize)]
pub enum EdgarArch {
    #[serde(rename="x86_64-unknown-linux-gnu")]
    X86_64,
    #[serde(rename="armv7-unknown-linux-gnueabihf")]
    Armhf,
    #[serde(rename="aarch64-unknown-linux-gnu")]
    Arm64,
}
impl EdgarArch {
    pub fn distribution_name(&self) -> String {
        match self {
            EdgarArch::X86_64 => "opendut-edgar-x86_64-unknown-linux-gnu",
            EdgarArch::Armhf => "opendut-edgar-armv7-unknown-linux-gnueabihf",
            EdgarArch::Arm64 => "opendut-edgar-aarch64-unknown-linux-gnu",
        }.to_string()
    }

    pub fn arch_iterator() -> Iter<'static, EdgarArch> {
        static EDGAR_ARCH: [EdgarArch; 3] = [EdgarArch::X86_64, EdgarArch::Armhf, EdgarArch::Arm64];
        EDGAR_ARCH.iter()
    }
}