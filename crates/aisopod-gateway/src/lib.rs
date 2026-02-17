//! # aisopod-gateway
//!
//! API gateway functionality, request routing, and external interface management.

pub mod auth;
pub mod middleware;
pub mod routes;
pub mod rpc;
pub mod server;
pub mod ws;

pub use server::run;
