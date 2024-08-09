use std::cell::RefCell;
use anyhow::Error;
use wasmtime::component::{Component, ResourceTable};
use wasmtime::{ Store};
use wasmtime_wasi::{DirPerms, FilePerms, WasiCtx, WasiCtxBuilder, WasiView};

use super::plugin_bindgen::SetupPlugin;
use super::plugin_bindgen::exports::edgar::setup::task::{Success, TaskFulfilled};
use super::task::Task;

pub struct SetupPluginStore {
    store:RefCell<Option<Store<PluginState>>>,
    component:Component,
    instance: SetupPlugin
}

impl SetupPluginStore{
    pub fn new(store: Store<PluginState>, component:Component, instance: SetupPlugin)->Self{
        Self{
            store: RefCell::new(Some(store)),
            component,
            instance
        }
    }
}

impl Task for SetupPluginStore{
    fn description(&self) -> String {
        let mut store = self.store.borrow_mut().take().unwrap();
        let result = self.instance.edgar_setup_task().call_description(&mut store).expect("Plugin could not call_description");
        *self.store.borrow_mut() = Some(store);
        result
    }

    fn check_fulfilled(&self) -> anyhow::Result<super::task::TaskFulfilled> {
        let mut store = self.store.borrow_mut().take().unwrap();
        let result = match self.instance.edgar_setup_task().call_check_fulfilled(&mut store).expect("Plugin could not call_check_fulfilled") {
            Ok(fulfilled) => {Ok(fulfilled.into())}
            Err(e) => {Err(Error::msg("Plugin check_fulfilled returned err"))}
        };
        *self.store.borrow_mut() = Some(store);
        result
    }

    fn execute(&self) -> anyhow::Result<super::task::Success> {
        let mut store = self.store.borrow_mut().take().unwrap();
        let result = match self.instance.edgar_setup_task().call_execute(&mut store).expect("Plugin could not call_execute"){
            Ok(success) => {Ok(success.into())}
            Err(e) => {Err(Error::msg("Plugin execute returned err"))}
        };
        *self.store.borrow_mut() = Some(store);
        result
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

impl From<TaskFulfilled> for super::task::TaskFulfilled{
    fn from(value: TaskFulfilled) -> Self {
        match value {
            TaskFulfilled::Yes => {super::task::TaskFulfilled::Yes}
            TaskFulfilled::No => {super::task::TaskFulfilled::No}
            TaskFulfilled::Unchecked => {super::task::TaskFulfilled::Unchecked}
        }
    }
}

impl From<Success> for super::task::Success{
    fn from(value: Success) -> Self {
        match value.message{
            Some(msg) => super::task::Success::message(msg),
            None => super::task::Success{message:None}
        }
    }
}
