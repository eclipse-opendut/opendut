pub mod bindings {
    use wit_bindgen::generate;

    generate!({path: "wit/world.wit", pub_export_macro: true, export_macro_name: "export"});
}

pub use crate::plugin::bindings::edgar::setup::host;
pub use crate::plugin::bindings::export;
pub use crate::plugin::bindings::exports::edgar::setup::task;
