# Issue 164: Implement node.describe and node.invoke RPC Methods

## Summary
Implement the `node.describe` and `node.invoke` RPC methods that allow paired devices to advertise their capabilities (camera, location, calendar, contacts, etc.) and allow the server to invoke those capabilities on demand.

## Location
- Crate: `aisopod-gateway`
- File: `crates/aisopod-gateway/src/rpc/node_capabilities.rs`

## Current Behavior
No mechanism exists for devices to report what services they offer or for the server to invoke device-side functionality remotely.

## Expected Behavior
1. **`node.describe`** — A paired device sends its list of capabilities after connecting. The server stores these capabilities in the connection state so agents can query what a device can do.
2. **`node.invoke`** — The server sends an invocation request to a device for a specific service method. The device executes the action locally and returns the result.

## Impact
This is the core of the app protocol — it turns every paired device into an extensible tool that agents can use. Without this, agents cannot access device hardware or native APIs.

## Suggested Implementation
1. Create `crates/aisopod-gateway/src/rpc/node_capabilities.rs`.
2. Define the capability types:
   ```rust
   use serde::{Deserialize, Serialize};

   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct DeviceCapability {
       pub service: String,           // e.g. "camera", "location", "calendar"
       pub methods: Vec<String>,      // e.g. ["take_photo", "record_video"]
       pub description: Option<String>,
   }

   #[derive(Debug, Deserialize)]
   pub struct NodeDescribeParams {
       pub capabilities: Vec<DeviceCapability>,
   }

   #[derive(Debug, Serialize)]
   pub struct NodeDescribeResult {
       pub accepted: bool,
       pub registered_services: Vec<String>,
   }
   ```
3. Define the invocation types:
   ```rust
   use serde_json::Value;

   #[derive(Debug, Serialize)]
   pub struct NodeInvokeRequest {
       pub service: String,
       pub method: String,
       pub params: Value,             // Arbitrary JSON params for the method
       pub timeout_ms: u64,
   }

   #[derive(Debug, Deserialize)]
   pub struct NodeInvokeResult {
       pub success: bool,
       pub data: Option<Value>,
       pub error: Option<String>,
   }
   ```
4. Implement `node.describe` handler:
   ```rust
   pub async fn handle_node_describe(
       connection: &mut ConnectionState,
       params: NodeDescribeParams,
   ) -> Result<NodeDescribeResult, RpcError> {
       let services: Vec<String> = params
           .capabilities
           .iter()
           .map(|c| c.service.clone())
           .collect();

       connection.device_capabilities = Some(params.capabilities);

       Ok(NodeDescribeResult {
           accepted: true,
           registered_services: services,
       })
   }
   ```
5. Implement `node.invoke` handler (server → client direction):
   - Look up the target device's connection by device ID.
   - Verify the requested service and method exist in that device's capabilities.
   - Send the `NodeInvokeRequest` as a JSON-RPC request to the device's WebSocket.
   - Await the response with a timeout.
   - Return the `NodeInvokeResult` to the calling agent.
6. Register both methods with the RPC router.
7. Add validation: reject `node.describe` from unpaired devices (error `-32003`), reject `node.invoke` for services the device did not declare.

## Dependencies
- Issue 030 (RPC router for method registration)
- Issue 033 (client connection management / ConnectionState)

## Acceptance Criteria
- [ ] `node.describe` accepts a capability list and stores it in connection state
- [ ] `node.invoke` routes an invocation to the correct device and returns its response
- [ ] Invoking an undeclared service/method returns an appropriate error
- [ ] Unpaired devices cannot call `node.describe`
- [ ] Invocations respect the configured timeout
- [ ] Unit tests cover describe, invoke success, invoke errors, and timeout

---
*Created: 2026-02-15*
