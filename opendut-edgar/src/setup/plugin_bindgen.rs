use wasmtime::component::bindgen;

use super::task::Task;

#[cfg(target_os="windows")]
bindgen!({world: "setup-plugin", path: ".\\src\\setup\\wit\\world.wit"});

#[cfg(not(target_os="windows"))]
bindgen!({world: "setup-plugin", path: "./src/setup/wit/world.wit"});
