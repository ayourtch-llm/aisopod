mod error;
mod message;
mod types;

pub mod client;

pub use client::{build_auth_request, AisopodClient};
pub use error::{ClientError, Result};
pub use message::{error_codes, error_response, parse_response, RpcRequest, RpcResponse};
pub use types::{AuthRequest, AuthResponse, ClientConfig, ClientState, DeviceCapability, DeviceInfo, PairRequestResult, PairConfirmResult, PairRevokeResult, ServerEvent, NodeDescribeResult, NodeInvokeResult, ChatResponse};
