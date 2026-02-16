# Issue 138: Update UI API Endpoints and WebSocket Client for Aisopod Gateway

## Summary
Update all API endpoint URLs and the WebSocket client in the rebranded UI to connect to the aisopod gateway server, including auth flow and JSON-RPC method names.

## Location
- Crate: N/A (frontend)  
- File: `ui/src/` (API client modules, WebSocket client)

## Current Behavior
The copied UI still references OpenClaw gateway API routes, WebSocket URLs, and possibly different JSON-RPC method names. The UI cannot communicate with the aisopod backend.

## Expected Behavior
The UI connects to the aisopod gateway via WebSocket using the correct endpoint URLs, authentication flow, and JSON-RPC method names. Reconnection logic handles dropped connections gracefully.

## Impact
Without this change the UI cannot communicate with the backend at all. This is critical for any UI functionality.

## Suggested Implementation

1. **Locate the API client configuration** (typically a config or constants file):
   ```bash
   grep -r 'ws://' ui/src/ --include='*.ts'
   grep -r 'wss://' ui/src/ --include='*.ts'
   grep -r '/api/' ui/src/ --include='*.ts'
   ```

2. **Update the WebSocket connection URL:**
   ```typescript
   // Before (example)
   const WS_URL = 'ws://localhost:8080/ws';

   // After — use relative URL so it works behind any host/port
   const WS_URL = `ws://${window.location.host}/ws`;
   ```

3. **Update the authentication flow** if the gateway uses a different auth mechanism:
   ```typescript
   // Ensure the WebSocket connection sends the correct auth token
   const ws = new WebSocket(WS_URL);
   ws.onopen = () => {
     ws.send(JSON.stringify({
       jsonrpc: '2.0',
       method: 'auth.authenticate',
       params: { token: getAuthToken() },
       id: 1,
     }));
   };
   ```

4. **Update JSON-RPC method names** to match aisopod conventions:
   ```typescript
   // Example method name mapping
   // Old: 'agent.list' → New: 'agents.list' (if changed)
   // Review all method calls against aisopod gateway RPC definitions
   ```

5. **Verify reconnection logic** handles disconnects:
   ```typescript
   ws.onclose = (event) => {
     if (!event.wasClean) {
       setTimeout(() => connectWebSocket(), 3000);
     }
   };
   ```

6. **Update any REST API endpoints** if the UI uses HTTP requests alongside WebSocket:
   ```typescript
   // Update base URL for any fetch() calls
   const API_BASE = '/api/v1';
   ```

## Dependencies
- Issue 137 (copy and rebrand UI)
- Issue 026 (gateway server implementation)
- Issue 029 (JSON-RPC support)

## Acceptance Criteria
- [ ] WebSocket client connects to aisopod gateway successfully
- [ ] Authentication flow completes without errors
- [ ] JSON-RPC method names match aisopod gateway definitions
- [ ] Reconnection logic recovers from dropped connections within 5 seconds
- [ ] No hardcoded OpenClaw-specific API URLs remain
- [ ] All REST API endpoint URLs updated to aisopod routes

---
*Created: 2026-02-15*
