//! Static file serving for the web UI
//!
//! This module provides embedded static file serving using `rust-embed`.
//! It handles SPA routing by returning index.html for unknown paths,
//! sets appropriate MIME types, and configures cache headers.

use axum::{
    extract::State,
    http::{header, StatusCode, Uri},
    response::IntoResponse,
};
use rust_embed::{Embed, RustEmbed};
use std::sync::Arc;
use tokio::sync::RwLock;

use aisopod_config::types::WebUiConfig;

/// Embedded static assets from the web UI dist directory
///
/// The assets are embedded at compile time from the `web-ui/dist` directory
/// which should be at the workspace root.
#[derive(RustEmbed)]
#[folder = "../../web-ui/dist"]
struct Assets;

/// Static file serving state
#[derive(Clone)]
pub struct StaticFileState {
    config: Arc<RwLock<WebUiConfig>>,
}

impl StaticFileState {
    pub fn new(config: WebUiConfig) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
        }
    }

    pub async fn get_config(&self) -> WebUiConfig {
        self.config.read().await.clone()
    }
}

/// Determine the MIME type based on file extension
pub fn get_content_type(path: &str) -> &'static str {
    match path.rsplit_once('.') {
        Some((_, ext)) => match ext.to_lowercase().as_str() {
            "html" | "htm" => "text/html; charset=utf-8",
            "css" => "text/css; charset=utf-8",
            "js" => "application/javascript; charset=utf-8",
            "json" => "application/json; charset=utf-8",
            "png" => "image/png",
            "jpg" | "jpeg" => "image/jpeg",
            "gif" => "image/gif",
            "svg" => "image/svg+xml; charset=utf-8",
            "ico" => "image/x-icon",
            "webp" => "image/webp",
            "woff" => "font/woff",
            "woff2" => "font/woff2",
            "ttf" => "font/ttf",
            "otf" => "font/otf",
            "txt" => "text/plain; charset=utf-8",
            _ => "application/octet-stream",
        },
        None => "application/octet-stream",
    }
}

/// Check if a filename contains a hash segment (for cache control)
fn has_hash_in_filename(path: &str) -> bool {
    // Look for hash patterns like .abc123def or abc123def456 in the filename
    let filename = path.rsplit('/').next().unwrap_or(path);
    // Hash segments are typically 8+ characters of alphanumeric or hyphen
    // Pattern: filename.abc123.ext or filename.abc123def456.ext
    // Accept alphanumeric chars (base64-like) for hash detection
    filename
        .split('.')
        .any(|part| {
            part.len() >= 8
                && part
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        })
}

/// Determine cache control header value based on file type
pub fn get_cache_control(path: &str) -> &'static str {
    if path == "index.html" || path.ends_with("/index.html") {
        "no-cache"
    } else if has_hash_in_filename(path) {
        // Immutable assets with hashes get long-lived caching
        "public, max-age=31536000, immutable"
    } else {
        // Non-hashed assets should be revalidated
        "public, max-age=0, must-revalidate"
    }
}

/// Static file handler (internal)
///
/// Serves static files from embedded assets or returns index.html for SPA fallback.
/// API routes take precedence over static files.
async fn static_handler_internal(state: StaticFileState, path: &str) -> impl IntoResponse {
    // Check if static files are enabled
    let config = state.get_config().await;
    if !config.enabled {
        return (StatusCode::NOT_FOUND, "Not Found").into_response();
    }

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
            headers.insert(
                header::CONTENT_TYPE,
                content_type
                    .parse()
                    .expect("content_type is a valid header value"),
            );
            headers.insert(
                header::CACHE_CONTROL,
                cache_control
                    .parse()
                    .expect("cache_control is a valid header value"),
            );

            (headers, file.data).into_response()
        }
        None => {
            // SPA fallback: return index.html for unknown paths
            if path.is_empty() || path == "index.html" {
                if let Some(file) = Assets::get("index.html") {
                    let content_type = get_content_type("index.html");
                    let cache_control = get_cache_control("index.html");

                    let mut headers = axum::http::HeaderMap::new();
                    headers.insert(
                        header::CONTENT_TYPE,
                        content_type
                            .parse()
                            .expect("content_type is a valid header value"),
                    );
                    headers.insert(
                        header::CACHE_CONTROL,
                        cache_control
                            .parse()
                            .expect("cache_control is a valid header value"),
                    );

                    (headers, file.data).into_response()
                } else {
                    (StatusCode::INTERNAL_SERVER_ERROR, "index.html not found").into_response()
                }
            } else {
                // For non-API paths that don't match a file, return index.html
                // This enables SPA routing
                if let Some(file) = Assets::get("index.html") {
                    let content_type = get_content_type("index.html");
                    let cache_control = get_cache_control("index.html");

                    let mut headers = axum::http::HeaderMap::new();
                    headers.insert(
                        header::CONTENT_TYPE,
                        content_type
                            .parse()
                            .expect("content_type is a valid header value"),
                    );
                    headers.insert(
                        header::CACHE_CONTROL,
                        cache_control
                            .parse()
                            .expect("cache_control is a valid header value"),
                    );

                    (headers, file.data).into_response()
                } else {
                    (StatusCode::NOT_FOUND, "Not Found").into_response()
                }
            }
        }
    }
}

/// Static file handler for use with axum routing
pub async fn static_handler(State(state): State<StaticFileState>, uri: Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/');
    static_handler_internal(state, path).await
}

/// Build CORS headers based on configured origins
pub fn build_cors_headers(origins: &[String]) -> String {
    origins.join(", ")
}
