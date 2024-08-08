use wasmtime::component::{Component, ResourceTable};
use wasmtime::{ Store};
use wasmtime_wasi::{DirPerms, FilePerms, WasiCtx, WasiCtxBuilder, WasiView};

use super::plugin_bindgen::SetupPlugin;
use super::task::Task;



pub struct SetupPluginStore {
    store:Store<PluginState>,
    component:Component,
    instance: SetupPlugin
}

impl SetupPluginStore{
    pub fn new(store: Store<PluginState>, component:Component, instance: SetupPlugin)->Self{
        Self{
            store,
            component,
            instance
        }
    }
}

impl Task for SetupPluginStore{
    fn description(&mut self) -> String {
        self.instance.edgar_setup_task().call_description(&mut self.store)
    }

    fn check_fulfilled(&mut self) -> anyhow::Result<super::task::TaskFulfilled> {
        self.instance.edgar_setup_task().call_check_fulfilled(&mut self.store)
    }

    fn execute(&mut self) -> anyhow::Result<super::task::Success> {
        self.instance.edgar_setup_task().call_execute(&mut self.store)
    }
}

pub struct PluginState {
    ctx: WasiCtx,
    table: ResourceTable,
}

impl PluginState{
    pub fn new()-> Self{
        let mut ctx_builder = WasiCtxBuilder::new();
        ctx_builder.inherit_stdio();
        ctx_builder.preopened_dir("/", "/", DirPerms::all(), FilePerms::all());
        ctx_builder.inherit_network();

        Self{
            ctx: ctx_builder.build(),
            table: ResourceTable::new()
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
