use anyhow::Result;
use axum::{response::IntoResponse, routing::get, Json, Router};
use serde_json::json;
use std::net::SocketAddr;
use tracing::{info, warn};
use tokio::signal;
use tower_http::trace::{TraceLayer, DefaultMakeSpan};
use tracing::Level;

use aisopod_config::types::GatewayConfig;
use crate::routes::api_routes;
use crate::ws::ws_routes;

/// Health check endpoint handler
async fn health() -> impl IntoResponse {
    Json(json!({"status": "ok"}))
}

/// Run the Axum HTTP server with the given configuration
pub async fn run(config: &GatewayConfig) -> Result<()> {
    let address = config.bind.address.clone();
    let port = config.server.port;

    let bind_addr = if config.bind.ipv6 {
        format!("[{}]:{}", address, port)
    } else {
        format!("{}:{}", address, port)
    };

    let addr: SocketAddr = bind_addr
        .parse()
        .map_err(|e| anyhow::anyhow!("Failed to parse bind address '{}': {}", bind_addr, e))?;

    info!("Starting HTTP server on {}", addr);

    // Create a tokio TCP listener for the server
    let tcp_listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to bind to address '{}': {}", addr, e))?;

    // Build the router with the /health endpoint, API routes, and WebSocket route
    let app = Router::new()
        .route("/health", get(health))
        .merge(ws_routes())
        .merge(api_routes())
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
        );

    // Set up graceful shutdown signal
    let shutdown_signal = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");

        warn!("Received SIGINT, starting graceful shutdown...");
    };

    // Optionally register SIGTERM handler as well
    let mut sigterm_stream = signal::unix::signal(signal::unix::SignalKind::terminate())
        .expect("Failed to install SIGTERM handler");

    let sigterm_signal = async move {
        sigterm_stream.recv().await;
        warn!("Received SIGTERM, starting graceful shutdown...");
    };

    // Run the server with graceful shutdown
    let server = axum::serve(tcp_listener, app);

    // Use select to handle either signal
    let server_with_graceful = server.with_graceful_shutdown(async {
        tokio::select! {
            _ = shutdown_signal => {},
            _ = sigterm_signal => {},
        }
    });

    server_with_graceful.await?;

    info!("Server shut down gracefully");
    Ok(())
}
