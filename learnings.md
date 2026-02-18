# Learnings Log

## Conversation Summary
The team resolved failing integration tests in `aisopod-gateway` by fixing middleware ordering (especially `AuthConfigData` and `ConnectInfo` injection), resolving duplicate test modules, and improving WebSocket response handling—resulting in all 16 tests initially passing. However, intermittent failures re-emerged under parallel test execution (`--test-threads=8`), primarily affecting WebSocket tests (`test_ws_connect_and_ping`, `test_malformed_json_returns_parse_error`, `test_valid_rpc_request`), likely due to shared state (e.g., `RateLimiter`, `ClientRegistry`) not being reset between tests. Debug output showed tests receiving unexpected `Ping([])` instead of JSON-RPC responses, indicating race conditions or resource leakage between tests sharing ports/middleware state. Recent test runs show mixed results: some runs pass fully, others fail intermittently, confirming flakiness.

## Key Technical Details

### Tests Affected by Parallel Execution
- `test_ws_connect_and_ping`
- `test_malformed_json_returns_parse_error`
- `test_valid_rpc_request`

### Suspected Causes of Flakiness
1. **Shared state not reset between tests:**
   - `RateLimiter`
   - `ClientRegistry`
2. **Resource leakage between tests**
3. **Tests sharing ports/middleware state**
4. **Race conditions in async code**

### Debug Observations
- Tests receiving unexpected `Ping([])` instead of JSON-RPC responses
- Indicates race conditions or resource leakage

### Previous Fixes Applied
- Fixed middleware ordering (`AuthConfigData` and `ConnectInfo` injection)
- Resolved duplicate test modules
- Improved WebSocket response handling
- All 16 tests initially passed after fixes
