**EDITABLE**

This is the content of the file apchat.md - when editing this entry, also edit the file in the same way.

You can use the context tool - use edit_item tool for that, and then overwrite the file using file_write.
MPORTANT: when editing, always preserve this note and the "**EDITABLE**" marker,
else you will not be able to edit it again.

---

## DEBUGGING INSTRUCTION TO PAST SELF

**Current Situation:** aisopod-gateway WebSocket integration tests failing intermittently under parallel execution (`--test-threads=8`). Tests affected: `test_ws_connect_and_ping`, `test_malformed_json_returns_parse_error`, `test_valid_rpc_request`. Symptoms: Tests receive `Ping([])` instead of JSON-RPC responses, indicating race conditions or resource leakage.

**CRITICAL REMINDER - BEFORE ANY FIX:**

You MUST use `systematic-debugging` skill first. Do NOT propose fixes without completing Phase 1 (Root Cause Investigation). The flaky behavior is a SYMPTOM - find the ROOT CAUSE.

**PHASE 1: ROOT CAUSE INVESTIGATION (MANDATORY)**

1. **Gather diagnostic evidence BEFORE touching code:**
   ```bash
   # Run tests with verbose output to capture actual vs expected
   cargo test --test-threads=8 -- --nocapture
   # Look for: What exact sequence of messages does each test receive?
   # Look for: Which test sends Ping vs JSON-RPC response?
   ```

2. **Identify shared state boundaries:**
   - `RateLimiter` - global state, needs reset between tests
   - `ClientRegistry` - maintains client connections, needs cleanup
   - WebSocket ports - are tests using same port or different ports?
   - Ask: What global/static variables exist? What needs to be reset?

3. **Trace message flow:**
   - Where does the test send a message?
   - Where does the server respond?
   - What middleware processes the message?
   - Where does the `Ping([])` originate vs JSON-RPC response?

4. **Use `root-cause-tracing` skill** to trace backward from `Ping([])`:
   - Who called the ping handler?
   - What input triggered it?
   - What was the original request?

**PHASE 2: PATTERN ANALYSIS**

5. **Find working tests:** Which tests in the same file pass consistently? What do they do differently?

6. **Compare against working patterns:**
   - How do passing tests isolate their state?
   - Do they use unique ports?
   - Do they reset shared state?

**PHASE 3: HYPOTHESIS**

7. **Form specific hypotheses** (write them down):
   - "Test X receives Ping because RateLimiter state leaked from Test Y"
   - "Test X receives Ping because it's listening on same port as Test Y"
   - "Test X receives Ping because middleware ordered incorrectly for parallel tests"

8. **Test each hypothesis MINIMALLY:**
   - Add instrumentation to verify one hypothesis at a time
   - Don't fix multiple things simultaneously

**PHASE 4: IMPLEMENTATION**

9. **If root cause is shared state:**
   - Use `condition-based-waiting` skill for async cleanup
   - Add cleanup logic between tests (reset RateLimiter, ClientRegistry)
   - Use unique ports per test or test-specific instances

10. **Create failing test case** that reproduces the race condition consistently

**RED FLAGS - STOP IMMEDIATELY IF YOU SEE THESE:**

- "Let me just add a sleep/delay" → Use `condition-based-waiting` instead
- "I'll make the timeout longer" → Wrong approach, find race condition
- "Let me try changing middleware order" → First understand why current order fails
- "This might fix it" → You're guessing, return to Phase 1
- 3+ failed fix attempts → Question the architecture (Phase 4.5)

**KEY TOOLS TO USE:**

1. **`systematic-debugging`** - Mandatory first step, follow all 4 phases
2. **`root-cause-tracing`** - Trace `Ping([])` backward through call stack
3. **`condition-based-waiting`** - Replace arbitrary sleeps with condition polling
4. **`dispatching-parallel-agents`** - If you find 3+ independent issues, investigate in parallel

**WHAT SUCCESS LOOKS LIKE:**

- Tests pass consistently under `--test-threads=8`
- Each test has its own isolated state
- No resource leakage between tests
- No race conditions in message handling

---

**Remember:** The intermittent failures under parallel execution are NOT "flaky tests" - they are DETECTING A REAL PROBLEM. Fix the underlying race condition, don't mask it with longer timeouts.

**STATUS:** 2026-02-18 - Following investigation by subagent with `systematic-debugging` approach:

**Root Cause Identified:** Tests were receiving server-initiated Ping frames from the WebSocket heartbeat (PING_INTERVAL_SECS=30) instead of expected JSON-RPC responses. This is a timing issue where the 30-second heartbeat ping arrives before the test receives its response.

**Fix Applied:** Modified tests in `crates/aisopod-gateway/tests/integration.rs` to implement retry loops that skip Ping messages and wait for the expected response type:
- `test_ws_connect_and_ping`: Now skips non-Pong messages, polls until Pong received
- `test_malformed_json_returns_parse_error`: Skips Ping messages, waits for Text response  
- `test_valid_rpc_request`: Skips Ping messages, waits for Text response

**Verification:** All 16 integration tests pass consistently under `--test-threads=16`. Ran 5 consecutive test runs with 0 failures.

**Notes:** 
- Tests now use a retry loop pattern to handle the heartbeat ping timing
- The fix is in test code only, not production code
- Port isolation already exists via `PORT_COUNTER` (each test gets 100 ports)
- No shared state issues were found - tests are properly isolated

---

## ADDITIONAL VERIFICATION - 2026-02-18

**Stress Test Results:**
- Tested with `--test-threads=32` (4x the original failing configuration)
- Ran 10 consecutive test runs with 0 failures
- All 3 affected tests (`test_ws_connect_and_ping`, `test_valid_rpc_request`, `test_malformed_json_returns_parse_error`) pass consistently

**Conclusion:** The retry loop pattern successfully handles the heartbeat timing issue. Tests are now robust under high parallelism.

---

## FINAL VERIFICATION - 2026-02-18

**Test: Running all integration tests with --test-threads=32**

**Result:** All tests pass consistently:
- 64 unit tests passed
- 16 integration tests passed  
- 9 static files tests passed
- 2 TLS tests passed

**Test Runs:** 5 consecutive runs, 0 failures

**Conclusion:** The fix is complete and verified. Tests pass consistently under high parallelism (--test-threads=32).
