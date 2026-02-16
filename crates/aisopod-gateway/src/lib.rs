//! # aisopod-gateway
//!
//! API gateway functionality, request routing, and external interface management.

pub mod routes;
pub mod server;

pub use server::run;
