pub mod bindings {
    use wit_bindgen::generate;

    generate!({path: "wit/world.wit", pub_export_macro: true, export_macro_name: "export"});
}

pub use crate::bindings::export;
pub use crate::bindings::exports::edgar::setup::task;

pub mod wit{
    pub const WIT: &str = include_str!("../wit/world.wit");
}
