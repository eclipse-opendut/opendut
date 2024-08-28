pub use ior::Ior;
pub use tick::Tick;

pub mod url;
mod ior;
mod tick;
pub mod net;
pub mod view;
pub mod clipboard;

pub const NON_BREAKING_SPACE: &str = "\u{a0}";
