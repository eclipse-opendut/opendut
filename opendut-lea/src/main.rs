use leptos::*;
use tracing::info;
use tracing_subscriber::fmt::format::Pretty;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

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
mod nav;
mod user;
mod about;

fn main() {

    console_error_panic_hook::set_once();

    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_ansi(false)
        .without_time()
        .with_writer(tracing_web::MakeConsoleWriter)
        .pretty();
    let perf_layer = tracing_web::performance_layer()
        .with_details_from_fields(Pretty::default());

    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(perf_layer)
        .init();

    info!("LEA started.");

    mount_to_body(|| view! { <App /> })
}
