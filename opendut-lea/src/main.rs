#![allow(clippy::empty_docs)]  //Temporarily ignore these, as they are currently buggy: https://github.com/rust-lang/rust-clippy/issues/12377 (as of 2024-05-14)

use leptos::prelude::*;
use leptos_router::components::Router;
use tracing::info;
use tracing_subscriber::fmt::format::Pretty;
use tracing_subscriber::{filter, Layer};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use crate::app::LoadingApp;

mod app;
mod peers;
mod api;
mod dashboard;
mod components;
mod clusters;
mod error;
mod routing;
mod licenses;
mod nav;
mod user;
mod about;
mod downloads;
mod util;

fn main() {

    console_error_panic_hook::set_once();

    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_ansi(false)
        .without_time()
        .with_writer(tracing_web::MakeConsoleWriter)
        .pretty()
        .with_filter(
            filter::Targets::default()
                .with_default(tracing::Level::DEBUG)
                .with_target("opendut", tracing::Level::TRACE)
        );

    let perf_layer = tracing_web::performance_layer()
        .with_details_from_fields(Pretty::default());

    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(perf_layer)
        .init();

    info!("LEA started.");

    mount_to_body(|| view! {
        <Router>
            <LoadingApp />
        </Router>
    })
}
