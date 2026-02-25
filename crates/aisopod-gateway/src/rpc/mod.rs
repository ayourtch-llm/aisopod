//! JSON-RPC 2.0 types and parsing functionality

pub mod approval;
pub mod chat;
pub mod handler;
pub mod middleware;
pub mod types;

pub use handler::{default_router, MethodRouter, PlaceholderHandler, RequestContext, RpcMethod, ApprovalRequestHandler, ApprovalApproveHandler, ApprovalDenyHandler, ApprovalListHandler};
pub use approval::{PendingApproval, ApprovalStatus, ApprovalRequestParams, ApprovalStore};
pub use types::{error_codes, parse, RpcError, RpcRequest, RpcResponse};

pub mod jsonrpc {
    pub use crate::rpc::types::{error_codes, RpcError, RpcResponse};
}
