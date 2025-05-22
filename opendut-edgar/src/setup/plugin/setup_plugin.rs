use std::sync::{Arc, Mutex};
use std::thread;
use anyhow::Error;
use async_trait::async_trait;
use wasmtime::Store;
use crate::setup::plugin::plugin_runtime::PluginState;
use opendut_edgar_plugin_api::host::SetupPlugin;
use crate::common::task::Task;

pub struct SetupPluginStore {
    store: Arc<Mutex<Store<PluginState>>>,
    instance: Arc<Mutex<SetupPlugin>>,
}

impl SetupPluginStore {
    pub fn new(store: Store<PluginState>, instance: SetupPlugin) -> Self {
        Self {
            store: Arc::new(Mutex::new(store)),
            instance: Arc::new(Mutex::new(instance))
        }
    }
}

#[async_trait]
impl Task for SetupPluginStore {
    fn description(&self) -> String {
        let store = self.store.clone();
        let instance = self.instance.clone();

        let thread = thread::spawn(move || {
            let mut store = store.lock().unwrap();
            let instance = instance.lock().expect("Unable to lock mutex");

            instance.edgar_setup_task().call_description(&mut *store).expect("Plugin could not call_description")
        });

        thread.join().expect("Failed to join thread for reading description")
    }

    async fn check_present(&self) -> anyhow::Result<crate::common::task::TaskStateFulfilled> {
        let store = self.store.clone();
        let instance = self.instance.clone();

        let thread = thread::spawn(move || {
            let mut store = store.lock().unwrap();
            let instance = instance.lock().expect("Unable to lock mutex");

            instance.edgar_setup_task().call_check_fulfilled(&mut *store).expect("Plugin could not call_check_fulfilled")
        });

        let result = thread.join().expect("Failed to join thread for reading description");

        match result {
            Ok(task_fulfilled) => Ok(task_fulfilled.into()),
            Err(_) => Err(Error::msg("Plugin check_fulfilled() returned with Err")),
        }
    }

    async fn make_present(&self) -> anyhow::Result<crate::common::task::Success> {
        let store = self.store.clone();
        let instance = self.instance.clone();
        
        let thread = thread::spawn(move || {
            let mut store = store.lock().unwrap();
            let instance = instance.lock().expect("Unable to lock mutex");

            instance.edgar_setup_task().call_execute(&mut *store).expect("Plugin could not call_execute")
        });

        let result = thread.join().expect("Failed to join thread for reading description");

        match result {
            Ok(task_fulfilled) => Ok(task_fulfilled.into()),
            Err(_) => Err(Error::msg("Plugin execute() returned with Err")),
        }
    }
}
