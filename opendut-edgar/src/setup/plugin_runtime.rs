use super::setup_plugin::SetupPluginStore;
use opendut_edgar_plugin_api::host::{Host, LogLevel, SetupPlugin, Success, TaskFulfilled};
use std::path::{Path, PathBuf};
use std::process::Command;
use wasmtime::component::__internal;
use wasmtime::component::{Component, Linker, ResourceTable};
use wasmtime::{Config, Engine, Store};
use wasmtime_wasi::{DirPerms, FilePerms, WasiCtx, WasiCtxBuilder, WasiView};
use tracing::{event, trace, Level};

pub struct PluginRuntime {
    config: Config,
    engine: Engine,
    linker: Linker<PluginState>,
}

impl PluginRuntime {
    pub fn new() -> Self {
        let mut config = Config::new();
        config.wasm_component_model(true);
        config.debug_info(true);

        let engine = Engine::new(&config).unwrap();

        let mut linker = Linker::new(&engine);

        wasmtime_wasi::add_to_linker_sync(&mut linker).expect("Could not add wasi to linker");

        // Necessary if Interface defines Imports
        SetupPlugin::add_to_linker(&mut linker, |state: &mut PluginState| state)
            .expect("Could not add PluginState to linker");

        Self {
            config,
            engine,
            linker,
        }
    }

    pub fn create_plugin_from_wasm(&self, plugin_path: impl AsRef<Path>) -> SetupPluginStore {
        let mut store = Store::new(&self.engine, PluginState::new(&plugin_path));

        let component = Component::from_file(&self.engine, plugin_path).unwrap();

        let instance = SetupPlugin::instantiate(&mut store, &component, &self.linker)
            .expect("Could not instantiate plugin");

        SetupPluginStore::new(store, component, instance)
    }
}

pub struct PluginState {
    path: PathBuf,
    ctx: WasiCtx,
    table: ResourceTable,
}

impl PluginState {
    pub fn new(path: &impl AsRef<Path>) -> Self {
        let mut ctx_builder = WasiCtxBuilder::new();
        ctx_builder.inherit_stdio();
        ctx_builder
            .preopened_dir("/", "/", DirPerms::all(), FilePerms::all())
            .expect("Could not preopen folder for plugin");
        ctx_builder.inherit_network();

        Self {
            ctx: ctx_builder.build(),
            table: ResourceTable::new(),
            path: path.as_ref().to_path_buf()
        }
    }
}

impl WasiView for PluginState {
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.table
    }

    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.ctx
    }
}

impl Host for PluginState {
    fn call_command(
        &mut self,
        command: __internal::String,
        args: __internal::Vec<__internal::String>,
    ) -> Result<__internal::String, __internal::String> {
        trace!("Plugin executing command {} with args {:?}", command, args);


        let mut cmd = Command::new(command);
        cmd.args(args);

        let result = cmd.output();
        trace!("Plugin command resulted in {:?}", result);
        match result {
            Ok(output) => Ok(String::from_utf8(output.stdout).expect("Command output could not be converted to String")),
            Err(e) => Err(e.to_string()),
        }
    }
    
    fn log(&mut self,level:LogLevel, message:__internal::String,) -> () {
        let path = self.path.to_str().expect("Path invalid unicode");
        match level {
            LogLevel::Trace => event!(Level::TRACE, plugin = path, message),
            LogLevel::Debug => event!(Level::DEBUG, plugin = path, message),
            LogLevel::Info => event!(Level::INFO, plugin = path, message),
            LogLevel::Warn => event!(Level::WARN, plugin = path, message),
            LogLevel::Error => event!(Level::ERROR, plugin = path, message),
        }
    }

    
}

impl From<TaskFulfilled> for super::task::TaskFulfilled {
    fn from(value: TaskFulfilled) -> Self {
        match value {
            TaskFulfilled::Yes => super::task::TaskFulfilled::Yes,
            TaskFulfilled::No => super::task::TaskFulfilled::No,
            TaskFulfilled::Unchecked => super::task::TaskFulfilled::Unchecked,
        }
    }
}

impl From<Success> for super::task::Success {
    fn from(value: Success) -> Self {
        match value.message {
            Some(msg) => super::task::Success::message(msg),
            None => super::task::Success { message: None },
        }
    }
}
