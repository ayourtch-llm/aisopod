# Generalized Meta-Learnings

## Overview

This document captures meta-patterns and generalized approaches for debugging and fixing test failures, applicable across any project and any parallel execution scenario. These patterns emerge from investigating intermittent test failures but apply broadly to any debugging scenario.

---

## Pattern 1: Heartbeat/Background Task Interference

### Problem
Tests expecting specific responses receive background/heartbeat messages instead.

### Root Cause
Server-side background tasks (heartbeats, keep-alive, health checks) send periodic messages that arrive before the test's expected response.

### Diagnosis
1. Check server code for periodic tasks (timers, intervals)
2. Trace message flow: where do unexpected messages originate?
3. Look for constant-time delays in server code

### Solution
```rust
// Test-side fix: add retry loop to skip intermediate messages
let mut msg = ws.next().await.unwrap();
while !matches!(msg, ExpectedMessageType) {
    msg = ws.next().await.unwrap();  // Skip intermediate messages
}
assert!(matches!(msg, ExpectedMessageType));
```

### Generalization
- **Any server with background tasks** can interfere with tests
- Always add retry loops in tests when expecting specific messages
- The loop should skip all messages that are NOT the expected type

---

## Pattern 2: Port/Resource Reuse Conflicts

### Problem
Tests fail with port bind errors or "connection refused" intermittently under parallel execution.

### Root Cause
Tests reuse the same ports before previous tests have fully cleaned up their resources.

### Diagnosis
1. Look for static counters used for port allocation
2. Check if ports are being released before server shutdown completes
3. Run tests individually - if they pass, but fail in parallel, it's a resource conflict

### Solution
```rust
// Give each test unique port range
static PORT_COUNTER: AtomicU16 = AtomicU16::new(30000);
fn find_available_port() -> u16 {
    PORT_COUNTER.fetch_add(100, SeqCst) + 10000  // 100 ports per test
}

// Wait for port to be available before using it
fn wait_for_port_release(port: u16) {
    for _ in 0..100 {
        if TcpListener::bind(("127.0.0.1", port)).is_ok() {
            return;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}
```

### Generalization
- **Any test using network resources** (ports, sockets) needs isolation
- Each test should get its own resource range, not share
- Always wait for resource cleanup before reusing

---

## Pattern 3: Shared State Leakage

### Problem
Tests affect each other's behavior, causing unexpected results.

### Root Cause
Global/static variables or shared mutable state not reset between tests.

### Diagnosis
1. Find all global/static variables
2. Check if any state persists between tests
3. Run tests in isolation vs. in parallel - if behavior differs, state is shared

### Solution
```rust
// Reset shared state between tests
fn reset_shared_state() {
    RateLimiter::reset();
    ClientRegistry::reset();
    // Reset any other global state
}

#[tokio::test]
async fn test_something() {
    reset_shared_state();
    // ... rest of test
}
```

### Generalization
- **Any test using shared mutable state** needs explicit reset
- Use `before_each`/`after_each` patterns where available
- Document all global state and how it's managed

---

## Pattern 4: Async Cleanup Race Conditions

### Problem
Tests pass with `--test-threads=1` but fail under parallel execution.

### Root Cause
Async resources (channels, connections, tasks) not properly closed before test ends, causing cleanup to happen during subsequent test.

### Diagnosis
1. Check all async resources are properly dropped
2. Look for tokio tasks that might outlive the test
3. Add debug output to trace cleanup timing

### Solution
```rust
// Use condition-based waiting for async cleanup
fn wait_for_cleanup(condition: impl Fn() -> bool) {
    for _ in 0..100 {
        if condition() {
            return;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}

// Drop resources explicitly
drop(ws_tx);  // Drop sender to signal receiver to close
drop(ws_rx);  // Drop receiver
wait_for_cleanup(|| !is_resource_in_use());
```

### Generalization
- **Any async test** needs explicit resource cleanup
- Always drop async resources before test ends
- Use condition-based waiting, not arbitrary sleeps

---

## Pattern 5: Message Ordering in Async Systems

### Problem
Tests receive messages in unexpected order or receive wrong messages.

### Root Cause
Multiple async tasks sending messages concurrently, messages arrive out of order or get buffered.

### Diagnosis
1. Trace all message sources in the system
2. Add debug logging to see message sequence
3. Check if tests assume specific ordering that isn't guaranteed

### Solution
```rust
// Test should not assume message order
let mut messages = Vec::new();
while messages.len() < expected_count {
    let msg = ws.next().await.unwrap();
    if matches!(msg, ExpectedType) {
        messages.push(msg);
    }
}
// Now process messages in any order
```

### Generalization
- **Any async test** should not assume message ordering
- Tests should accumulate expected messages, not assume order
- Use pattern matching to filter relevant messages

---

## Testing Best Practices

### 1. Always Test Under Parallel Execution
```bash
# Test at 4x expected parallelism
cargo test -- --test-threads=$((N * 4))
```

### 2. Run Multiple Times to Catch Intermittent Failures
```bash
for i in $(seq 1 10); do cargo test ...; done
```

### 3. Use Unique Resources Per Test
- Ports: Give each test 100+ port range
- Files: Use unique temp directories
- Databases: Use unique database names

### 4. Implement Proper Cleanup
- Drop async resources explicitly
- Wait for cleanup completion
- Use `Drop` implementations for automatic cleanup

### 5. Add Debug Output for Failing Tests
```rust
eprintln!("=== TEST RECEIVED: {:?} ===", msg);
```
This helps diagnose failures in CI where you can't use a debugger.

---

## Debugging Checklist

When tests fail intermittently under parallel execution:

- [ ] **Step 1**: Gather raw evidence with `--nocapture`
- [ ] **Step 2**: Trace unexpected messages backward
- [ ] **Step 3**: Identify all shared resources
- [ ] **Step 4**: Check test isolation (run individually)
- [ ] **Step 5**: Add retry loops for async messages
- [ ] **Step 6**: Verify cleanup with stress tests
- [ ] **Step 7**: Run 5+ times to confirm fix

---

## Key Principles

1. **Intermittent failures detect real issues** - Don't mask with longer timeouts
2. **Parallel execution exposes race conditions** - Always test under parallelism
3. **Tests should be independent** - No shared state between tests
4. **Async requires explicit cleanup** - Don't rely on GC or drop timing
5. **Debug output is essential** - Can't use debugger in CI/test environments
