pub mod bindings {
    use wit_bindgen::generate;

    generate!({path: "wit/world.wit", pub_export_macro: true, export_macro_name: "export"});
}

pub use crate::bindings::export;
pub use crate::bindings::exports::edgar::setup::task;
pub use crate::bindings::edgar::setup::host;
