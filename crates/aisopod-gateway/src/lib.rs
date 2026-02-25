//! # aisopod-gateway
//!
//! API gateway functionality, request routing, and external interface management.

pub mod audit;
pub mod auth;
pub mod broadcast;
pub mod client;
pub mod middleware;
pub mod routes;
pub mod rpc;
pub mod server;
pub mod static_files;
pub mod tls;
pub mod ws;

pub use server::run;
pub use server::run_with_config;
pub use server::build_app;
pub use routes::{GatewayStatus, GatewayStatusState};
