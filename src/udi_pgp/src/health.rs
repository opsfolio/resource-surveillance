use axum::{http::StatusCode, routing::get, Json, Router};
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, process};
use tokio::sync::oneshot;
use tracing::{error, info, warn};

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

    match tokio::net::TcpListener::bind(addr).await {
        Ok(listener) => {
            info!("Health server is binding on {}", addr);
            match axum::serve(listener, app)
                .with_graceful_shutdown(graceful_shutdown(shutdown_signal))
                .await
            {
                Ok(_) => info!("Health server running on {}", addr),
                Err(e) => {
                    error!("Server error: {}", e);
                    return Err(e.into()); // Make sure to return the error for further handling if needed
                }
            }
        }
        Err(e) => {
            error!("Failed to bind health server on {}: {}", addr, e);
            return Err(e.into());
        }
    }

    Ok(())
}

async fn graceful_shutdown(shutdown_signal: oneshot::Receiver<()>) {
    match shutdown_signal.await {
        Ok(()) => info!("Health server has received shutdown signal."),
        Err(err) => warn!("Health server shutdown signal dropped: {}", err),
    }
}

async fn get_health() -> (StatusCode, Json<HealthResponseData>) {
    let res = HealthResponseData {
        status: "pass".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        description: env!("CARGO_PKG_DESCRIPTION").to_string(),
        service_id: process::id().to_string(),
    };
    (StatusCode::OK, Json(res))
}
