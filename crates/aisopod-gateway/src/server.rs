use anyhow::Result;
use axum::{
    body::Body,
    extract::State,
    http::{header, StatusCode, Uri},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde_json::json;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tracing::{info, warn};
use tokio::signal;
use tower_http::trace::{TraceLayer, DefaultMakeSpan, DefaultOnResponse};
use tower_http::cors::{CorsLayer, Any};
use tracing::Level;

use aisopod_config::types::{AisopodConfig, GatewayConfig};
use crate::routes::api_routes;
use crate::ws::ws_routes;
use crate::middleware::{auth_middleware, AuthConfigData, RateLimiter, RateLimitConfig, rate_limit_middleware};
use crate::client::ClientRegistry;
use crate::broadcast::Broadcaster;
use crate::static_files::{StaticFileState, get_content_type, get_cache_control};
use rust_embed::RustEmbed;

/// Embedded static assets from the web UI dist directory
#[derive(RustEmbed)]
#[folder = "../../web-ui/dist"]
struct Assets;

/// Health check endpoint handler
async fn health() -> impl IntoResponse {
    Json(json!({"status": "ok"}))
}

/// Static file handler for use with axum routing
async fn static_file_handler(
    State(state): State<StaticFileState>,
    uri: Uri,
) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/');
    
    // Skip API routes (they are handled by the router)
    if path.starts_with("api/") || path.starts_with("v1/") || path.starts_with("ws") {
        return (StatusCode::NOT_FOUND, "Not Found").into_response();
    }

    // Try to get the file from embedded assets
    match Assets::get(path) {
        Some(file) => {
            let content_type = get_content_type(path);
            let cache_control = get_cache_control(path);
            
            let mut headers = axum::http::HeaderMap::new();
            headers.insert(header::CONTENT_TYPE, content_type.parse().unwrap());
            headers.insert(header::CACHE_CONTROL, cache_control.parse().unwrap());
            
            (headers, file.data).into_response()
        }
        None => {
            // SPA fallback: return index.html for unknown paths
            if let Some(file) = Assets::get("index.html") {
                let content_type = get_content_type("index.html");
                let cache_control = get_cache_control("index.html");
                
                let mut headers = axum::http::HeaderMap::new();
                headers.insert(header::CONTENT_TYPE, content_type.parse().unwrap());
                headers.insert(header::CACHE_CONTROL, cache_control.parse().unwrap());
                
                (headers, file.data).into_response()
            } else {
                (StatusCode::INTERNAL_SERVER_ERROR, "index.html not found").into_response()
            }
        }
    }
}

/// Run the Axum HTTP server with the given configuration
pub async fn run_with_config(config: &AisopodConfig) -> Result<()> {
    let gateway_config = &config.gateway;
    let auth_config = &config.auth;
    
    let address = gateway_config.bind.address.clone();
    let port = gateway_config.server.port;

    let bind_addr = if gateway_config.bind.ipv6 {
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

    // Create rate limiter with configuration from GatewayConfig
    let config_rate_limit = &gateway_config.rate_limit;
    let rate_limit_config = RateLimitConfig::new(
        config_rate_limit.max_requests,
        Duration::from_secs(config_rate_limit.window),
    );
    let rate_limiter = Arc::new(RateLimiter::new(rate_limit_config.clone()));
    
    // Spawn the cleanup task
    let cleanup_limiter = rate_limiter.clone();
    tokio::spawn(async move {
        cleanup_limiter.cleanup_loop().await;
    });
    
    // Build the auth config data
    let auth_config_data = AuthConfigData::new(auth_config.clone());
    
    // Create the client registry
    let client_registry = Arc::new(ClientRegistry::new());
    
    // Create the broadcast channel for gateway events
    let broadcaster = Arc::new(Broadcaster::new(128));
    
    // Setup static file serving state
    let web_ui_config = gateway_config.web_ui.clone();
    let static_state = StaticFileState::new(web_ui_config.clone());
    
    // Build the router with the /health endpoint, API routes, and WebSocket route
    // Use the configured handshake timeout or default to 5 seconds
    let handshake_timeout = if gateway_config.handshake_timeout > 0 {
        Some(gateway_config.handshake_timeout)
    } else {
        None
    };
    
    // Create a router for static files that comes before API routes
    let static_router = Router::new()
        .route("/", get(static_file_handler))
        .route("/*path", get(static_file_handler))
        .with_state(static_state);
    
    let app = Router::new()
        .route("/health", get(health))
        .nest_service("/", static_router)
        .merge(ws_routes(handshake_timeout))
        .merge(api_routes())
        .layer(axum::middleware::from_fn(move |mut req: axum::extract::Request, next: axum::middleware::Next| {
            let config_data = auth_config_data.clone();
            async move {
                req.extensions_mut().insert(config_data);
                next.run(req).await
            }
        }))
        .layer(axum::middleware::from_fn(move |mut req: axum::extract::Request, next: axum::middleware::Next| {
            let rate_limiter = rate_limiter.clone();
            async move {
                req.extensions_mut().insert(rate_limiter);
                next.run(req).await
            }
        }))
        .layer(axum::middleware::from_fn(move |mut req: axum::extract::Request, next: axum::middleware::Next| {
            let registry = client_registry.clone();
            async move {
                req.extensions_mut().insert(registry);
                next.run(req).await
            }
        }))
        .layer(axum::middleware::from_fn(move |mut req: axum::extract::Request, next: axum::middleware::Next| {
            let broadcaster = broadcaster.clone();
            async move {
                req.extensions_mut().insert(broadcaster);
                next.run(req).await
            }
        }))
        .layer(axum::middleware::from_fn(auth_middleware))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_response(
                    DefaultOnResponse::new().level(Level::INFO)
                )
        )
        .layer(axum::middleware::from_fn(rate_limit_middleware));

    // Build CORS layer based on web UI configuration
    let cors_origins = web_ui_config.cors_origins;
    let cors = if cors_origins.is_empty() {
        CorsLayer::permissive()
    } else {
        use tower_http::cors::Any;
        CorsLayer::new()
            .allow_origin(cors_origins.iter().map(|s| s.parse().unwrap()).collect::<Vec<_>>())
            .allow_methods(tower_http::cors::Any)
            .allow_headers(tower_http::cors::Any)
    };
    
    let app = app.layer(cors);

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

/// Run the Axum HTTP server with the given configuration (backward compatible)
pub async fn run(config: &GatewayConfig) -> Result<()> {
    let aisopod_config = AisopodConfig {
        gateway: config.clone(),
        ..Default::default()
    };
    run_with_config(&aisopod_config).await
}
