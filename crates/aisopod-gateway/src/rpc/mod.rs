//! JSON-RPC 2.0 types and parsing functionality

pub mod approval;
pub mod canvas;
pub mod chat;
pub mod handler;
pub mod middleware;
pub mod node_capabilities;
pub mod node_pair;
pub mod types;

pub use approval::{ApprovalRequestParams, ApprovalStatus, ApprovalStore, PendingApproval};
pub use canvas::{
    CanvasAction, CanvasContent, CanvasInteractParams, CanvasInteractResult, CanvasState,
    CanvasUpdateParams,
};
pub use handler::{
    default_router, ApprovalApproveHandler, ApprovalDenyHandler, ApprovalListHandler,
    ApprovalRequestHandler, CanvasInteractHandler, MethodRouter, PlaceholderHandler,
    RequestContext, RpcMethod,
};
pub use node_capabilities::{
    CapabilityStore, NodeDescribeHandler, NodeDescribeParams, NodeDescribeResult,
    NodeInvokeHandler, NodeInvokeRequest, NodeInvokeResult,
};
pub use node_pair::{
    generate_pairing_code, run_pairing_cleanup_task, PairConfirmHandler, PairConfirmParams,
    PairConfirmResult, PairRequestHandler, PairRequestParams, PairRequestResult, PairRevokeHandler,
    PairRevokeParams, PairRevokeResult, PairingStore, PendingPairing,
};
pub use types::{error_codes, parse, RpcError, RpcRequest, RpcResponse};

// Re-export DeviceCapability from client module
pub use crate::client::DeviceCapability;

pub mod jsonrpc {
    pub use crate::rpc::types::{error_codes, RpcError, RpcResponse};
}
