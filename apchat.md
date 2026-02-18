**EDITABLE**

This is the content of the file apchat.md - when editing this entry, also edit the file in the same way.

You can use the context tool - use edit_item tool for that, and then overwrite the file using file_write.
MPORTANT: when editing, always preserve this note and the "**EDITABLE**" marker,
else you will not be able to edit it again.

---

## GENERALIZED DEBUGGING FRAMEWORK FOR INTERMITTENT TEST FAILURES

### PROBLEM CLASSIFICATION
Intermittent test failures under parallel execution indicate one or more of these root causes:

| Symptom | Most Likely Root Cause | Investigation Focus |
|---------|----------------------|---------------------|
| Tests receive unexpected Ping frames | Heartbeat timing conflicts | Message flow analysis |
| Tests receive data from other tests | Shared state leakage | Global/static variables |
| Port conflicts between tests | Port reuse without cleanup | Port allocation strategy |
| Intermittent timeouts | Race conditions in async code | Async cleanup patterns |

### INVESTIGATION PROCESS

**PHASE 1: DIAGNOSTIC GATHERING**
```bash
# Gather raw evidence before touching code
cargo test --test-threads=N -- --nocapture

# Look for patterns:
# - Which messages do tests actually receive vs expect?
# - What is the sequence of messages per test?
# - Which tests fail together (indicating shared state issues)?
```

**PHASE 2: ROOT CAUSE IDENTIFICATION**
1. **Trace the unexpected message backward** - where does it originate?
2. **Identify all shared resources** - global vars, statics, ports
3. **Check test isolation** - can tests run individually without failure?
4. **Verify async cleanup** - are channels, connections properly closed?

**PHASE 3: FIX STRATEGY**
- **Heartbeat timing**: Add retry loops to skip expected-intermediate messages
- **Shared state**: Add reset/cleanup between tests
- **Port conflicts**: Use unique port ranges per test (N ports per test)
- **Async cleanup**: Use condition-based waiting, not arbitrary delays

### META-PATTERNS

**Pattern 1: Heartbeat Interference**
```
Symptom: Tests receive Ping frames instead of expected messages
Root: Server heartbeat sends periodic Ping frames
Fix: Add retry loops to skip Ping frames, poll for expected message type
```

**Pattern 2: Port Reuse Conflicts**
```
Symptom: Port bind failures, connection refused
Root: Tests reuse ports before previous server fully shuts down
Fix: Give each test unique port range + wait_for_port_release helper
```

**Pattern 3: Async Cleanup Race**
```
Symptom: Intermittent failures that disappear with --test-threads=1
Root: Async resources not cleaned up before next test starts
Fix: Use condition-based waiting for cleanup completion
```

### TESTING VERIFICATION

**Success Criteria:**
- Tests pass under `--test-threads=N` where N ≥ expected max parallelism
- 5+ consecutive runs with 0 failures
- Tests pass individually AND in parallel

**Stress Test Command:**
```bash
# Test at 4x expected parallelism
cargo test -- --test-threads=$((N * 4))

# Run multiple times to catch intermittent failures
for i in $(seq 1 10); do cargo test ...; done
```

---

**Remember:** Intermittent failures under parallel execution are NOT "flaky tests" - they are DETECTING REAL ISSUES. Fix the underlying race condition, don't mask it with longer timeouts.
