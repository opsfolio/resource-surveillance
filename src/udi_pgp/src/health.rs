use std::{net::SocketAddr, process};

use axum::{http::StatusCode, routing::get, Json, Router};
use serde::{Deserialize, Serialize};
use tokio::sync::oneshot;
use tracing::{error, info};

#[derive(Debug, Clone, Deserialize, Serialize)]
struct HealthResponseData {
    status: String,
    version: String,
    description: String,
    #[serde(rename(serialize = "serviceId"))]
    service_id: String,
}

pub async fn start(addr: SocketAddr, shutdown_signal: oneshot::Receiver<()>) -> anyhow::Result<()> {
    let app = Router::new().route("/health", get(get_health));

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    match axum::serve(listener, app)
        .with_graceful_shutdown(graceful_shutdown(shutdown_signal))
        .await
    {
        Ok(_) => info!("Health server running on {}", addr),
        Err(e) => error!("Server error: {}", e),
    }

    Ok(())
}

async fn graceful_shutdown(shutdown_signal: oneshot::Receiver<()>) {
    let st = shutdown_signal.await;
    if let Err(err) = st {
        error!("Failed to stop health server: {}", err);
    }
}

/// This handler serializes the health into a string for Prometheus to scrape
async fn get_health() -> (StatusCode, Json<HealthResponseData>) {
    let res = HealthResponseData {
        status: "pass".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        description: env!("CARGO_PKG_DESCRIPTION").to_string(),
        service_id: process::id().to_string(),
    };
    (StatusCode::OK, Json(res))
}
