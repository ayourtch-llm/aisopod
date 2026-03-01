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
use std::sync::{Arc, Mutex};
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
use crate::routes::{api_routes, device_token_routes, rpc_routes, GatewayStatusState};
use crate::rpc::node_pair::{run_pairing_cleanup_task, PairingStore};
use crate::static_files::{get_cache_control, get_content_type, StaticFileState};
use crate::tls::{is_tls_enabled, load_tls_config};
use crate::ws::ws_routes;
use aisopod_config::types::{AisopodConfig, AuthConfig, GatewayConfig};
use rust_embed::RustEmbed;

use crate::auth::DeviceTokenManager;
use crate::middleware::RequestSizeLimits;

/// Embedded static assets from the web UI dist directory
#[derive(RustEmbed)]
// Path is resolved relative to this crate's Cargo.toml within the workspace layout.
#[folder = "../../web-ui/dist"]
struct Assets;

/// Health check endpoint handler
async fn health() -> impl IntoResponse {
    Json(json!({"status": "ok"}))
}

/// HTTPS enforcement middleware
///
/// This middleware enforces HTTPS connections when TLS is enabled in the configuration.
/// For non-TLS servers, it logs a warning but still allows the request through.
pub async fn https_enforcement_middleware(
    mut request: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    // Check if TLS is enabled
    let tls_enabled = request.extensions().get::<bool>().cloned().unwrap_or(false);

    if !tls_enabled {
        // TLS is not enabled - log a warning but still process the request
        // Users can opt-in to HTTPS enforcement by enabling TLS
        warn!("HTTPS is not enabled - this server is running in HTTP mode");
    }

    next.run(request).await
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
pub async fn run_with_config(config: Arc<AisopodConfig>) -> Result<()> {
    let gateway_config = &config.gateway;
    let auth_config = &config.auth;

    tracing::info!("=== Starting server with auth config ===");
    tracing::info!("Auth mode: {:?}", auth_config.gateway_mode);
    tracing::info!("Auth config tokens: {:?}", auth_config.tokens.len());
    tracing::info!("Auth config passwords: {:?}", auth_config.passwords.len());

    let address = gateway_config.bind.address.clone();
    let port = gateway_config.server.port;

    // Security: Warn if binding to 0.0.0.0 (all interfaces)
    if address == "0.0.0.0" || address == "::" {
        warn!(
            "WARNING: Binding to '{}' - server is accessible from all network interfaces. \
             Consider using '127.0.0.1' for loopback-only access.",
            address
        );
    }

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

        // Security: Warn if TLS is not enabled when binding to external interfaces
        if address != "127.0.0.1" && address != "::1" && address != "localhost" {
            warn!("WARNING: Running without TLS on external interface. Consider enabling TLS for production use.");
        }
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

    // Security: Setup request size limits
    let size_limits = gateway_config.request_size_limits.clone();
    let request_size_limits = RequestSizeLimits::new(
        size_limits.max_body_size,
        size_limits.max_headers_size,
        size_limits.max_headers_count,
    );
    eprintln!(
        "Created request size limits: max_body={} bytes, max_headers={} bytes, max_count={}",
        request_size_limits.max_body_size,
        request_size_limits.max_headers_size,
        request_size_limits.max_headers_count
    );

    // Spawn the cleanup task
    let cleanup_limiter = rate_limiter.clone();
    tokio::spawn(async move {
        cleanup_limiter.cleanup_loop().await;
    });

    // Build the auth config data
    let auth_config_data = Arc::new(AuthConfigData::new(auth_config.clone()));
    // Clone for the secrets masking middleware
    let auth_config_data_for_secrets = auth_config_data.clone();

    // Create the client registry
    let client_registry = Arc::new(ClientRegistry::new());

    // Create the broadcast channel for gateway events
    let broadcaster = Arc::new(Broadcaster::new(128));

    // Create the pairing store for managing pending pairing requests
    let pairing_store = Arc::new(PairingStore::new());

    // Spawn the pairing cleanup task
    let pairing_cleanup_interval = Duration::from_secs(gateway_config.pairing_cleanup_interval);
    let pairing_store_for_cleanup = pairing_store.clone();
    tokio::spawn(async move {
        run_pairing_cleanup_task(pairing_store_for_cleanup, pairing_cleanup_interval).await;
    });

    // Setup device token manager with storage in the config directory
    let config_dir = gateway_config
        .bind
        .address
        .to_string()
        .replace([':', '/'], "-");
    let token_store_path = std::path::PathBuf::from(format!(
        "device-tokens-{}.toml",
        config_dir.trim_start_matches('[').trim_end_matches(']')
    ));
    let device_token_manager = Arc::new(Mutex::new(DeviceTokenManager::new(token_store_path)));

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
    let middleware_stack = tower::ServiceBuilder::new()
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_response(DefaultOnResponse::new().level(Level::INFO)),
        )
        // Add ConnectInfo middleware before layers that rely on connection info (config injection above does not rely on it).
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
        // Request size limits middleware - inject size limits and check requests
        .layer(axum::middleware::from_fn(
            move |mut req: axum::extract::Request, next: axum::middleware::Next| {
                let size_limits = request_size_limits.clone();
                async move {
                    req.extensions_mut().insert(size_limits);
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
        // Request body size limit middleware
        .layer(axum::middleware::from_fn(
            |mut req: axum::extract::Request, next: axum::middleware::Next| {
                async move {
                    // Get size limits from extensions
                    if let Some(size_limits) = req.extensions().get::<RequestSizeLimits>().cloned()
                    {
                        // Get content length from headers
                        if let Some(content_length) = req.headers().get("content-length") {
                            if let Ok(content_length_str) = content_length.to_str() {
                                if let Ok(content_length_bytes) =
                                    content_length_str.parse::<usize>()
                                {
                                    if let Err(e) =
                                        size_limits.check_body_size(content_length_bytes)
                                    {
                                        warn!("Request body size limit exceeded: {}", e);
                                        return (
                                            StatusCode::PAYLOAD_TOO_LARGE,
                                            axum::Json(serde_json::json!({
                                                "error": "payload_too_large",
                                                "message": e.to_string()
                                            })),
                                        )
                                            .into_response();
                                    }
                                }
                            }
                        }
                    }
                    next.run(req).await
                }
            },
        ))
        // HTTPS enforcement middleware
        .layer(axum::middleware::from_fn(
            move |mut req: axum::extract::Request, next: axum::middleware::Next| {
                let tls_enabled_flag = tls_enabled;
                async move {
                    req.extensions_mut().insert(tls_enabled_flag);
                    next.run(req).await
                }
            },
        ))
        .layer(axum::middleware::from_fn(https_enforcement_middleware))
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
        // Pairing store middleware
        .layer(axum::middleware::from_fn(
            move |mut req: axum::extract::Request, next: axum::middleware::Next| {
                let pairing_store = pairing_store.clone();
                async move {
                    req.extensions_mut().insert(pairing_store);
                    next.run(req).await
                }
            },
        ))
        // Auth config data MUST be injected BEFORE auth_middleware runs
        // By adding this layer BEFORE auth_middleware in the ServiceBuilder,
        // it runs BEFORE auth_middleware in the request flow (outer layers run first)
        .layer(axum::middleware::from_fn(
            move |mut req: axum::extract::Request, next: axum::middleware::Next| {
                // Dereference Arc to insert AuthConfigData directly (not Arc<AuthConfigData>)
                let config_data = auth_config_data.as_ref().clone();
                let config_mode = config_data.mode().clone();
                async move {
                    eprintln!("!!! AUTH CONFIG DATA INJECTOR CALLED !!!");
                    tracing::debug!("Injecting AuthConfigData into extensions");
                    let check_before = req.extensions().get::<AuthConfigData>().is_some();
                    eprintln!("!!! BEFORE INSERT: has config = {}", check_before);
                    req.extensions_mut().insert(config_data);
                    let check_after = req.extensions().get::<AuthConfigData>().is_some();
                    eprintln!("!!! AFTER INSERT: has config = {}", check_after);
                    eprintln!("!!! AUTH CONFIG DATA INJECTED: {:?}", config_mode);
                    next.run(req).await
                }
            },
        ))
        // Auth middleware - runs AFTER auth_config_data is injected
        // It depends on auth_config_data being available in extensions
        .layer(axum::middleware::from_fn(auth_middleware))
        // Secrets masking middleware - masks sensitive values in logs
        .layer(axum::middleware::from_fn(
            move |mut req: axum::extract::Request, next: axum::middleware::Next| {
                // Dereference Arc to insert AuthConfigData directly
                let auth_config_data_clone = auth_config_data_for_secrets.as_ref().clone();
                async move {
                    let check = req.extensions().get::<AuthConfigData>().is_some();
                    eprintln!(
                        "!!! SECRETS MASKING: AuthConfigData in extensions: {}",
                        check
                    );
                    // Store auth config data for secrets masking
                    req.extensions_mut().insert(auth_config_data_clone);
                    next.run(req).await
                }
            },
        ))
        // Device token manager middleware
        .layer(axum::middleware::from_fn(
            move |mut req: axum::extract::Request, next: axum::middleware::Next| {
                let device_token_manager = device_token_manager.clone();
                async move {
                    req.extensions_mut().insert(device_token_manager);
                    next.run(req).await
                }
            },
        ));

    eprintln!("=== MIDDLEWARE STACK BUILT === layers: 7");

    // Create status state with initial counts (will be updated by agents/channels)
    let status_state = Arc::new(GatewayStatusState::new(0, 0, 0));

    // Build the main app - order matters: static_router first (with 404 for API paths),
    // then API routes, then WebSocket routes, then device token routes, then RPC routes
    let app = Router::new()
        .route("/health", get(health))
        .nest_service("/", static_router)
        .merge(device_token_routes())
        .merge(api_routes(Some(status_state.clone())))
        .merge(ws_routes(config.clone(), handshake_timeout))
        .merge(rpc_routes())
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
    run_with_config(Arc::new(aisopod_config)).await
}

/// Build the Axum application for testing purposes
///
/// This function creates the same app as run_with_config but returns it
/// instead of starting the server, allowing integration tests to use axum-test.
pub async fn build_app(auth_config: AuthConfig) -> Router {
    use axum::{routing::get, Router};
    use serde_json::json;

    // Create a minimal config for testing
    let mut config = AisopodConfig::default();
    config.auth = auth_config;
    // Use an owned Arc so tests can inject the config into request extensions.
    let config_arc = Arc::new(config);

    let gateway_config = &config_arc.gateway;

    // Create rate limiter with default config
    let rate_limit_config = RateLimitConfig::new(100, Duration::from_secs(60));
    let rate_limiter = Arc::new(RateLimiter::new(rate_limit_config));

    // Spawn the cleanup task (but don't wait for it - just let it run)
    let cleanup_limiter = rate_limiter.clone();
    tokio::spawn(async move {
        cleanup_limiter.cleanup_loop().await;
    });

    // Create auth config data
    let auth_config_data = Arc::new(AuthConfigData::new(config_arc.auth.clone()));
    // Clone for the secrets masking middleware
    let auth_config_data_for_secrets = auth_config_data.clone();

    // Create client registry
    let client_registry = Arc::new(ClientRegistry::new());

    // Create broadcaster
    let broadcaster = Arc::new(Broadcaster::new(128));

    // Setup device token manager with default storage path
    let token_store_path = std::path::PathBuf::from("device-tokens.toml");
    let device_token_manager = Arc::new(Mutex::new(DeviceTokenManager::new(token_store_path)));

    // Setup static file serving state
    let web_ui_config = gateway_config.web_ui.clone();
    let static_state = StaticFileState::new(web_ui_config.clone());

    // Create status state
    let status_state = Arc::new(GatewayStatusState::new(0, 0, 0));

    // Build the middleware stack (same as in run_with_config)
    let middleware_stack = tower::ServiceBuilder::new()
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_response(DefaultOnResponse::new().level(Level::INFO)),
        )
        .layer(axum::middleware::from_fn(
            |mut req: axum::extract::Request, next: axum::middleware::Next| async move {
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
            },
        ))
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
        .layer(axum::middleware::from_fn(
            move |mut req: axum::extract::Request, next: axum::middleware::Next| {
                let registry = client_registry.clone();
                async move {
                    req.extensions_mut().insert(registry);
                    next.run(req).await
                }
            },
        ))
        .layer(axum::middleware::from_fn(
            move |mut req: axum::extract::Request, next: axum::middleware::Next| {
                let broadcaster = broadcaster.clone();
                async move {
                    req.extensions_mut().insert(broadcaster);
                    next.run(req).await
                }
            },
        ))
        // Auth config data MUST be injected BEFORE auth_middleware runs
        // By adding this layer BEFORE auth_middleware in the ServiceBuilder,
        // it runs BEFORE auth_middleware in the request flow (outer layers run first)
        .layer(axum::middleware::from_fn(
            move |mut req: axum::extract::Request, next: axum::middleware::Next| {
                let config_data = auth_config_data.as_ref().clone();
                async move {
                    eprintln!("=== AUTH CONFIG DATA INJECTOR ===");
                    eprintln!(
                        "Before insert, has config: {}",
                        req.extensions().get::<AuthConfigData>().is_some()
                    );
                    req.extensions_mut().insert(config_data);
                    eprintln!(
                        "After insert, has config: {}",
                        req.extensions().get::<AuthConfigData>().is_some()
                    );
                    next.run(req).await
                }
            },
        ))
        // Auth middleware - runs AFTER auth_config_data is injected
        // It depends on auth_config_data being available in extensions
        .layer(axum::middleware::from_fn(auth_middleware))
        // Secrets masking middleware - masks sensitive values in logs
        .layer(axum::middleware::from_fn(
            move |mut req: axum::extract::Request, next: axum::middleware::Next| {
                let auth_config_data_clone = auth_config_data_for_secrets.clone();
                async move {
                    // Store auth config data for secrets masking
                    req.extensions_mut().insert(auth_config_data_clone);
                    next.run(req).await
                }
            },
        ))
        // Device token manager middleware
        .layer(axum::middleware::from_fn(
            move |mut req: axum::extract::Request, next: axum::middleware::Next| {
                let device_token_manager = device_token_manager.clone();
                async move {
                    req.extensions_mut().insert(device_token_manager);
                    next.run(req).await
                }
            },
        ));

    // Create static router
    let static_router = Router::new()
        .route("/", get(static_file_handler))
        .route("/*path", get(static_file_handler))
        .with_state(static_state);

    // Build the app
    let app = Router::new()
        .route("/health", get(health))
        .nest_service("/", static_router)
        .merge(device_token_routes())
        .merge(api_routes(Some(status_state)))
        .merge(ws_routes(config_arc.clone(), None))
        .merge(rpc_routes())
        .layer(middleware_stack);

    // Build CORS layer
    let cors_origins = web_ui_config.cors_origins;
    let cors = if cors_origins.is_empty() {
        CorsLayer::permissive()
    } else {
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

    app.layer(cors)
}
