# Meta-Learnings: Testing Infrastructure Debugging

## Context
Debugging a failing test suite for `aisopod-provider` crate where `cargo test` fails with compilation errors after multiple changes to fix issues introduced by the implementation manager subagent.

## Pattern Observed: The Cascade of Test Failures

When an implementation subagent creates test files (like `discovery_tests.rs`, `normalize_tests.rs`, etc.) with compilation issues:

1. **Initial failure**: Tests fail to compile due to API mismatches
2. **Partial fix attempts**: Making targeted fixes introduces new issues
3. **State confusion**: Git checkout may not restore original state due to multiple modified paths
4. **Cascading errors**: One fix (like removing a field) reveals other hidden issues
5. **Test harness issues**: Missing integration test directories cause build failures

## Key Findings

### 1. Test Compilation vs. Library Compilation Are Separate Concerns
- `cargo build -p aisopod-provider` may succeed while `cargo test -p aisopod-provider` fails
- Tests have their own dependency chain and can expose issues not seen in library code
- **Always test both build targets separately**

### 2. The `ChatCompletionStream` Type Alias is Fragile
The `ChatCompletionStream` type alias in `trait_module.rs`:
```rust
pub type ChatCompletionStream =
    Pin<Box<dyn Stream<Item = Result<ChatCompletionChunk>> + Send>>;
```
This alias takes **one** generic parameter (`ChatCompletionChunk`), but the trait definition expects `Stream<Item = Result<T>>` which requires **two** generic parameters in some contexts.

### 3. `anyhow::Error` Requires `std::error::Error` Bound
The `anyhow::Error::new()` function requires the source error to implement `StdError + Send + Sync + 'static`. When passing `anyhow::Error` to itself, you get:
```
error[E0277]: the trait bound `anyhow::Error: std::error::Error` is not satisfied
```
**Solution**: Use `anyhow::anyhow!(source_error)` instead of `anyhow::Error::new(source_error)`.

### 4. Test Helper Files Are High-Maintenance
The `helpers/mod.rs` file:
- Defines `MockProvider` with `ChatCompletionStream` return type
- Uses `async_stream::stream!` macro for creating test streams
- Must be kept in sync with the trait definition
- Is a common source of compilation errors

### 5. Integration Test Configuration Is Tricky
The Cargo.toml test harness configuration:
```toml
[[test]]
name = "integration_tests"
path = "tests/integration/mod.rs"
harness = false
```
When `harness = false`, the file must contain a `main` function, which is often overlooked.

## Debugging Strategy That Works

### Step 1: Isolate the Problem
```bash
# Check if library builds
cargo build -p aisopod-provider

# Check if tests build (separate from library)
cargo test -p aisopod-provider --no-run
```

### Step 2: Examine Compilation Errors Systematically
```bash
# Get full error output
cargo test -p aisopod-provider 2>&1 | tee /tmp/test_errors.txt

# Count errors by type
grep "error\[" /tmp/test_errors.txt | cut -d'[' -f2 | cut -d']' -f1 | sort | uniq -c
```

### Step 3: Work Backward from Errors
1. Fix the FIRST error in the list (later errors may be cascading)
2. Rebuild after each fix
3. Check if new errors appear or old ones disappear

### Step 4: When All Else Fails - Fresh Start
```bash
# Restore original test files from git
git checkout crates/aisopod-provider/tests/

# Clean build artifacts
cargo clean -p aisopod-provider

# Rebuild
cargo build -p aisopod-provider
cargo test -p aisopod-provider
```

## Generalizable Principles

### Principle 1: Test Files Should Be Treated as Production Code
- They have compilation requirements
- They have type constraints
- They need to be maintained
- **Recommendation**: Keep test files minimal and extract shared logic

### Principle 2: Type Alias Changes Ripple Through Codebase
When you change a type alias:
- Find all uses of that type
- Update all implementations that use it
- Update all test files that use it
- **Recommendation**: Document type alias dependencies clearly

### Principle 3: Subagent-Generated Tests Often Miss Edge Cases
Subagents focused on implementation may:
- Forget to add necessary imports
- Use incorrect type signatures
- Miss trait bound requirements
- Create files in wrong locations
- **Recommendation**: Always have a verifier agent check test compilation

### Principle 4: Cargo.toml Test Configuration Is容易出错
Common pitfalls:
- Missing `harness = false` for integration tests
- Wrong file paths
- Missing `main` function when harness is false
- Duplicate test names
- **Recommendation**: Validate Cargo.toml test sections with `cargo metadata`

## Actionable Checklist for Future Debugging

When `cargo test` fails with compilation errors:

1. [ ] Check if `cargo build` succeeds
2. [ ] Run `cargo test --no-run` to see test compilation errors
3. [ ] Examine the FIRST compilation error in detail
4. [ ] Check type signatures against trait definitions
5. [ ] Verify imports in test files
6. [ ] Check Cargo.toml test configuration
7. [ ] Look for missing `main` function in non-harness tests
8. [ ] If multiple errors, fix one at a time and rebuild
9. [ ] If stuck, restore from git and start fresh
10. [ ] Document the root cause in learnings file

## Example: The `aggregate_usage` Fix Pattern

The `aggregate_usage` function signature mismatch:
```rust
// Function expects:
pub fn aggregate_usage(chunks: &[ChatCompletionChunk]) -> TokenUsage

// Test was passing:
let usages: Vec<TokenUsage> = chunks.iter().filter_map(...).collect();
aggregate_usage(&usages)  // WRONG: Wrong type
```

**Correct pattern**:
```rust
// Pass chunks directly, not extracted TokenUsage
let chunks: Vec<ChatCompletionChunk> = ...;
let result = aggregate_usage(&chunks);
```

## Meta-Meta: The Real Problem Was Not the Tests

The underlying issue wasn't that tests were failing - it was that:
1. The implementation introduced a breaking API change
2. Tests weren't updated to match the new API
3. The implementation manager subagent didn't have visibility into this
4. Multiple fix attempts created more problems than they solved

**Root cause**: API change without corresponding test updates.

**Prevention**: Add a CI check that validates test compilation on every PR.
