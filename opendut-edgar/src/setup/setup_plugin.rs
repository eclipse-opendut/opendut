use std::cell::RefCell;
use anyhow::Error;
use wasmtime::component::Component;
use wasmtime::Store;
use crate::setup::plugin_runtime::PluginState;
use super::plugin_bindgen::SetupPlugin;
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

