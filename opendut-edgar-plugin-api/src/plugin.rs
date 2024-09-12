pub mod bindings {
    use wit_bindgen::generate;

    generate!({path: "wit/world.wit", pub_export_macro: true, export_macro_name: "export"});

    pub fn trace(message: &str){
        edgar::setup::host::log(edgar::setup::host::LogLevel::Trace, message);
    }

    pub fn debug(message: &str){
        edgar::setup::host::log(edgar::setup::host::LogLevel::Debug, message);
    }

    pub fn info(message: &str){
        edgar::setup::host::log(edgar::setup::host::LogLevel::Info, message);
    }

    pub fn warn(message: &str){
        edgar::setup::host::log(edgar::setup::host::LogLevel::Warn, message);
    }

    pub fn error(message: &str){
        edgar::setup::host::log(edgar::setup::host::LogLevel::Error, message);
    }
}

impl crate::plugin::bindings::exports::edgar::setup::task::Success {
    pub fn new() -> Self {
        Self {
            message: None
        }
    }

    pub fn message(message: impl Into<String>) -> Self {
        Self { message: Some(message.into()) }
    }
}

pub use crate::plugin::bindings::{trace, debug, info, warn, error};
pub use crate::plugin::bindings::edgar::setup::host;
pub use crate::plugin::bindings::export;
pub use crate::plugin::bindings::exports::edgar::setup::task;
