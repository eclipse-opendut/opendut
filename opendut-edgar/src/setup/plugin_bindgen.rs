use wasmtime::component::bindgen;
use super::task::Task;

#[cfg(target_os="windows")]
bindgen!({world: "setup-plugin", path: "..\\opendut-edgar-plugin-api\\wit\\world.wit"});

#[cfg(not(target_os="windows"))]
bindgen!({world: "setup-plugin", path: "../opendut-edgar-plugin-api/wit/world.wit"});

pub use self::exports::edgar::setup::task::{Success, TaskFulfilled, Guest};
pub use self::edgar::setup::host::Host;
