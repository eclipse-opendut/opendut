/// General architecture used somewhere in the build process
#[derive(Clone, Copy, Debug)]
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
