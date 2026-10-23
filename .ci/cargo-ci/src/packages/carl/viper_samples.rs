use anyhow::Context;
use axum::{Router, http::header::CONTENT_TYPE, routing::get};
use std::{future::Future, io};
use tokio::{net::TcpListener, sync::oneshot};
use tracing::info;

use crate::{tasks::run::RunCli, workspace};

pub(super) async fn push_samples(cleo: RunCli) -> anyhow::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8000").await
        .context("Failed to bind VIPER sample server to 127.0.0.1:8000. Check whether the port is already in use.")?;

    serve_samples(
        listener,
        cleo.run_async(&workspace::package::opendut_cleo),
        tokio::signal::ctrl_c(),
    ).await
}

async fn serve_samples(
    listener: TcpListener,
    upload: impl Future<Output = anyhow::Result<()>>,
    shutdown_signal: impl Future<Output = io::Result<()>>,
) -> anyhow::Result<()> {
    let source_url = format!("http://{}/script.py", listener.local_addr()?);

    let app = Router::new().route(
        "/script.py",
        get(|| async {
            (
                [(CONTENT_TYPE, "text/x-python; charset=utf-8")],
                include_str!("viper_test_suite_sample.py"),
            )
        }),
    );

    let (shutdown_sender, shutdown_receiver) = oneshot::channel();
    let server = tokio::spawn(async move {
        axum::serve(listener, app)
            .with_graceful_shutdown(async move {
                let _ = shutdown_receiver.await;
            }).await
    });

    info!("Hosting VIPER sample source at {source_url}");

    let result = async {
        tokio::pin!(shutdown_signal);

        tokio::select! {
            result = upload => result?,
            signal = &mut shutdown_signal => {
                return signal.context("Failed to listen for Ctrl+C");
            }
        }

        info!("Sample data stored. VIPER source remains available at {source_url}. Press Ctrl+C to stop.");
        shutdown_signal.await.context("Failed to listen for Ctrl+C")
    }.await;

    info!("Stopping VIPER sample server");
    let _ = shutdown_sender.send(());
    server.await
        .context("Failed to join VIPER sample server task")?
        .context("VIPER sample server failed")?;

    result
}
