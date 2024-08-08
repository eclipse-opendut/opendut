use wasmtime::component::{bindgen, Component, Linker, ResourceTable};
use wasmtime::{Config, Engine, Store};
use wasmtime_wasi::{DirPerms, FilePerms, WasiCtx, WasiCtxBuilder, WasiView};

use super::plugin_bindgen::SetupPlugin;
use super::setup_plugin::{PluginState, SetupPluginStore};

pub struct PluginRuntime{
    config:Config,
    engine:Engine,
    linker:Linker<PluginState>
}

impl PluginRuntime{
    pub fn new()->Self{
        let mut config = Config::new();
        config.wasm_component_model(true);
        config.debug_info(true);

        let engine = Engine::new(&config).unwrap();

        let mut linker = Linker::new(&engine);

        wasmtime_wasi::add_to_linker_sync(&mut linker).expect("Could not add wasi to linker");
        SetupPlugin::add_to_linker(&mut linker, |state: &mut PluginState| state).expect("Could not add PluginState to linker");

        Self{
            config,
            engine,
            linker
        }
    }

    pub fn create_plugin_from_wasm(&self, plugin_path:impl AsRef<Path>)->SetupPluginStore{
        let mut store = Store::new(
            &self.engine, 
            PluginState::new()
        );

        let component = Component::from_file(&engine,plugin_path).unwrap();

        let instance = SetupPlugin::instantiate(&mut store, &component, &self.linker);

        SetupPluginStore::new(store, component, instance)
    }
}

