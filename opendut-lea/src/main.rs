use leptos::*;
use log::Level;

use crate::app::App;

mod app;
mod peers;
mod api;
mod dashboard;
mod components;
mod clusters;
mod error;
mod routing;
mod util;
mod licenses;


fn main() {
    let _ = console_log::init_with_level(Level::Debug);
    console_error_panic_hook::set_once();

    log::info!("LEA started.");

    mount_to_body(|| view! { <App /> })
}
