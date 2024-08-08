use wasmtime::component::bindgen;

#[cfg(target_os = "windows")]
bindgen!({world: "setup-plugin", path: ".\\wit\\world.wit"});

#[cfg(not(target_os = "windows"))]
bindgen!({world: "setup-plugin", path: "./wit/world.wit"});

pub use self::edgar::setup::host::Host;
pub use self::edgar::setup::host::LogLevel;
pub use self::exports::edgar::setup::task::{Guest, Success, TaskFulfilled};
