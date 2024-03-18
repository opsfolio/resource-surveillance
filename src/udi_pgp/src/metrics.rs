use std::net::SocketAddr;

use axum::{http::StatusCode, routing::get, Router};
use tokio::sync::oneshot;
use tracing::{error, info};

use autometrics::prometheus_exporter::encode_to_string;

pub async fn start(addr: SocketAddr, shutdown_signal: oneshot::Receiver<()>) -> anyhow::Result<()> {
    let app = Router::new().route("/health", get(get_metrics));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    match axum::serve(listener, app)
        .with_graceful_shutdown(graceful_shutdown(shutdown_signal))
        .await
    {
        Ok(_) => {
            info!("herreeeeeee");
            info!("Metrics server running on {}", addr)
        }
        Err(e) => error!("Server error: {}", e),
    }

    Ok(())
}

async fn graceful_shutdown(shutdown_signal: oneshot::Receiver<()>) {
    let st = shutdown_signal.await;
    if let Err(err) = st {
        error!("Failed to stop metrics server: {}", err);
    }
}

/// This handler serializes the health into a string for Prometheus to scrape
async fn get_metrics() -> (StatusCode, String) {
    match encode_to_string() {
        Ok(metrics) => (StatusCode::OK, metrics),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, format!("{:?}", err)),
    }
}
