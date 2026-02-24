#![allow(clippy::all)]
use anyhow::Result;
use axum::{
    body::Body,
    extract::{ConnectInfo, State},
    http::{header, StatusCode, Uri},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde_json::json;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::signal;
use tower::layer::util::Identity;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing::Level;
use tracing::{info, warn};

use crate::broadcast::Broadcaster;
use crate::client::ClientRegistry;
use crate::middleware::{
    auth_middleware, rate_limit_middleware, AuthConfigData, RateLimitConfig, RateLimiter,
};
use crate::routes::{api_routes, GatewayStatusState};
use crate::static_files::{get_cache_control, get_content_type, StaticFileState};
use crate::tls::{is_tls_enabled, load_tls_config};
use crate::ws::ws_routes;
use aisopod_config::types::{AisopodConfig, GatewayConfig};
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
async fn static_file_handler(State(state): State<StaticFileState>, uri: Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/');

    // Skip API routes (they are handled by the router)
    if path.starts_with("api/") || path.starts_with("v1/") || path.starts_with("ws") {
        return (StatusCode::NOT_FOUND, "Not Found").into_response();
    }

    // Check if static files are enabled
    let config = state.get_config().await;
    if !config.enabled {
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

    tracing::info!("=== Starting server with auth config ===");
    tracing::info!("Auth mode: {:?}", auth_config.gateway_mode);
    tracing::info!("Auth config tokens: {:?}", auth_config.tokens.len());
    tracing::info!("Auth config passwords: {:?}", auth_config.passwords.len());

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

    let tls_enabled = is_tls_enabled(&gateway_config.tls.cert_path, &gateway_config.tls.key_path);

    if tls_enabled {
        info!("Starting HTTPS server on {} with TLS enabled", addr);
    } else {
        info!("Starting HTTP server on {}", addr);
    }

    eprintln!("=== SERVER STARTUP DEBUG v2 ===");

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
    eprintln!(
        "Created rate limiter with max_requests = {}, window = {:?}",
        rate_limit_config.max_requests, rate_limit_config.window
    );

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

    // Build the middleware stack using Layer trait
    use tower::Layer;

    // Order matters! The middleware runs in reverse order (last layer listed runs first).
    // When a request comes in, it goes through layers from outside to inside.
    // We need auth_config_data AVAILABLE BEFORE auth_middleware runs.
    //
    // Request flow (outer to inner):
    // 1. TraceLayer (logs requests)
    // 2. ConnectInfo middleware (adds connection info if missing)
    // 3. auth_config_data middleware (injects auth config into extensions)
    // 4. rate_limiter middleware (checks rate limits)
    // 5. client_registry middleware (registers clients)
    // 6. broadcaster middleware (injects broadcaster)
    // 7. auth_middleware (validates auth - needs auth_config_data and rate_limiter)
    // 8. Router (handles the actual route)

    let middleware_stack = tower::ServiceBuilder::new()
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_response(DefaultOnResponse::new().level(Level::INFO)),
        )
        // Add ConnectInfo middleware - this must come before middleware that need it
        .layer(axum::middleware::from_fn(
            |mut req: axum::extract::Request, next: axum::middleware::Next| {
                async move {
                    // Add ConnectInfo if not already present (axum::serve adds it automatically)
                    // For local testing, we use 127.0.0.1 as the default connection info
                    let conn_info = req
                        .extensions()
                        .get::<ConnectInfo<SocketAddr>>()
                        .cloned()
                        .unwrap_or(ConnectInfo(std::net::SocketAddr::new(
                            std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST),
                            0,
                        )));
                    req.extensions_mut().insert(conn_info);
                    next.run(req).await
                }
            },
        ))
        // Auth config data MUST be injected BEFORE auth_middleware runs
        // This layer runs AFTER auth_middleware in the request flow (middleware stack order)
        .layer(axum::middleware::from_fn(
            move |mut req: axum::extract::Request, next: axum::middleware::Next| {
                let config_data = auth_config_data.clone();
                async move {
                    tracing::debug!("Injecting AuthConfigData into extensions");
                    req.extensions_mut().insert(config_data);
                    next.run(req).await
                }
            },
        ))
        // Rate limiter middleware - first insert the RateLimiter, then check limits
        .layer(axum::middleware::from_fn(
            move |mut req: axum::extract::Request, next: axum::middleware::Next| {
                let rate_limiter = rate_limiter.clone();
                async move {
                    req.extensions_mut().insert(rate_limiter);
                    next.run(req).await
                }
            },
        ))
        .layer(axum::middleware::from_fn(rate_limit_middleware))
        // Client registry middleware
        .layer(axum::middleware::from_fn(
            move |mut req: axum::extract::Request, next: axum::middleware::Next| {
                let registry = client_registry.clone();
                async move {
                    req.extensions_mut().insert(registry);
                    next.run(req).await
                }
            },
        ))
        // Broadcaster middleware
        .layer(axum::middleware::from_fn(
            move |mut req: axum::extract::Request, next: axum::middleware::Next| {
                let broadcaster = broadcaster.clone();
                async move {
                    req.extensions_mut().insert(broadcaster);
                    next.run(req).await
                }
            },
        ))
        // Auth middleware - runs FIRST (innermost in the stack, outermost in request flow)
        // It depends on auth_config_data being available in extensions
        .layer(axum::middleware::from_fn(auth_middleware));

    eprintln!("=== MIDDLEWARE STACK BUILT === layers: 7");
    
    // Create status state with initial counts (will be updated by agents/channels)
    let status_state = Arc::new(GatewayStatusState::new(0, 0, 0));
    
    // Build the main app - order matters: static_router first (with 404 for API paths),
    // then API routes, then WebSocket routes
    let app = Router::new()
        .route("/health", get(health))
        .nest_service("/", static_router)
        .merge(api_routes(Some(status_state.clone())))
        .merge(ws_routes(handshake_timeout))
        .layer(middleware_stack);

    // Build CORS layer based on web UI configuration
    let cors_origins = web_ui_config.cors_origins;
    let cors = if cors_origins.is_empty() {
        CorsLayer::permissive()
    } else {
        use tower_http::cors::Any;
        CorsLayer::new()
            .allow_origin(
                cors_origins
                    .iter()
                    .map(|s| s.parse().unwrap())
                    .collect::<Vec<_>>(),
            )
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

    // Use select to handle either signal
    let server_with_graceful = async {
        tokio::select! {
            _ = shutdown_signal => {},
            _ = sigterm_signal => {},
        }
    };

    if tls_enabled {
        // Start the server with TLS using axum-server
        let tls_config = load_tls_config(
            std::path::Path::new(&gateway_config.tls.cert_path),
            std::path::Path::new(&gateway_config.tls.key_path),
        )
        .await?;

        // Note: axum-server bind_rustls doesn't support with_graceful_shutdown
        // The server will shut down when the signal is received via Ctrl+C
        axum_server::bind_rustls(addr, tls_config)
            .serve(app.into_make_service())
            .await?;
    } else {
        // Start the server without TLS (plain HTTP)
        axum::serve(tcp_listener, app)
            .with_graceful_shutdown(server_with_graceful)
            .await?;
    }

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
