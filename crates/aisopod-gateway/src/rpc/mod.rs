//! JSON-RPC 2.0 types and parsing functionality

pub mod approval;
pub mod canvas;
pub mod chat;
pub mod handler;
pub mod middleware;
pub mod node_capabilities;
pub mod node_pair;
pub mod types;

pub use handler::{default_router, MethodRouter, PlaceholderHandler, RequestContext, RpcMethod, ApprovalRequestHandler, ApprovalApproveHandler, ApprovalDenyHandler, ApprovalListHandler, CanvasInteractHandler};
pub use approval::{PendingApproval, ApprovalStatus, ApprovalRequestParams, ApprovalStore};
pub use canvas::{CanvasState, CanvasUpdateParams, CanvasAction, CanvasContent, CanvasInteractParams, CanvasInteractResult};
pub use node_capabilities::{NodeDescribeHandler, NodeInvokeHandler, NodeDescribeParams, NodeDescribeResult, NodeInvokeRequest, NodeInvokeResult, CapabilityStore};
pub use node_pair::{PairingStore, PairRequestHandler, PairConfirmHandler, PairRevokeHandler, PairRequestParams, PairRequestResult, PairConfirmParams, PairConfirmResult, PairRevokeParams, PairRevokeResult, PendingPairing, generate_pairing_code, run_pairing_cleanup_task};
pub use types::{error_codes, parse, RpcError, RpcRequest, RpcResponse};

// Re-export DeviceCapability from client module
pub use crate::client::DeviceCapability;

pub mod jsonrpc {
    pub use crate::rpc::types::{error_codes, RpcError, RpcResponse};
}
