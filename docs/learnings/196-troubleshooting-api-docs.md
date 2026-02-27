# Issue 196: Troubleshooting Guide and API Documentation Setup

**Date:** 2026-02-27  
**Status:** Completed

## Summary

Implemented a comprehensive troubleshooting guide for Aisopod and configured automated Rust API documentation generation in CI.

## What Was Done

### 1. Created Comprehensive Troubleshooting Documentation

Created `docs/book/src/troubleshooting.md` with the following sections:

- **Common Errors** (8 errors documented with causes and solutions):
  - Connection refused (port 3080)
  - 401 Unauthorized
  - 502 Upstream Error
  - Channel connection failed
  - Sandbox execution timeout
  - Config parse error
  - Rate limit exceeded
  - Out of memory

- **Diagnostic Commands**:
  - `aisopod doctor` - comprehensive health check
  - Targeted checks with `--check-*` flags
  - Additional diagnostic commands for gateway, provider, and channel status

- **Log Analysis**:
  - Log level configuration (error, warn, info, debug, trace)
  - Key log patterns for troubleshooting
  - Structured JSON logging

- **Channel-Specific Troubleshooting**:
  - Telegram, Discord, WhatsApp, Slack-specific issues and solutions
  - General channel testing commands

- **Performance Tuning**:
  - Gateway configuration tuning
  - Memory usage optimization
  - Latency optimization techniques
  - Throughput scaling strategies

### 2. Updated CI Workflow

Modified `.github/workflows/ci.yml` to:
- Install Rust toolchain for docs job
- Generate Rust API docs with `cargo doc --workspace --no-deps`
- Combine mdBook and cargo doc output into a single artifact
- Upload combined documentation as a single artifact

### 3. Verification

All acceptance criteria verified:
- ✅ `docs/book/src/troubleshooting.md` exists and is linked from `SUMMARY.md`
- ✅ 8 common errors documented with causes and solutions (exceeds requirement of 6)
- ✅ `aisopod doctor` command and all `--check-*` flags documented
- ✅ Log level configuration and structured logging documented
- ✅ Channel-specific troubleshooting for all Tier 1 channels
- ✅ Performance tuning covers gateway, memory, latency, and throughput
- ✅ `cargo doc --workspace --no-deps` generates documentation without errors
- ✅ CI pipeline combines mdBook and cargo doc output into a single artifact
- ✅ `mdbook build` succeeds with this page included
- ✅ `cargo build` passes with `RUSTFLAGS=-Awarnings`

## Learnings

### Documentation Best Practices

1. **Structured troubleshooting guides** should follow a consistent pattern:
   - Problem statement (error message)
   - Root cause explanation
   - Concrete solution with commands
   - Verification steps

2. **Diagnostic commands documentation** should include:
   - The main diagnostic command
   - Output format examples
   - Targeted check options
   - What to look for in results

3. **Log analysis documentation** should cover:
   - Log level hierarchy and use cases
   - Common patterns to grep for
   - Structured logging format
   - Filtering and analysis techniques

### CI/CD Integration

1. **Documentation builds** should be separate from code builds:
   - Use dedicated workflow jobs
   - Cache dependencies appropriately
   - Combine outputs into a single artifact

2. **API documentation** should be:
   - Generated from source code
   - Versioned alongside releases
   - Linked from main documentation site

### Rust Documentation

1. **cargo doc** produces high-quality documentation:
   - All public items are documented
   - Intra-doc links work well
   - Code examples are rendered

2. **Common warnings** to fix later:
   - Broken intra-doc links
   - Unclosed HTML tags in doc comments
   - Bare URLs that should be automatic links

## Files Modified

1. `docs/book/src/troubleshooting.md` - Created comprehensive guide
2. `.github/workflows/ci.yml` - Added Rust API docs generation

## Files Created

1. `docs/learnings/196-troubleshooting-api-docs.md` - This file

## Follow-up Tasks

Based on this implementation, consider:

1. **Improve Rust doc comments** in the codebase to reduce warnings
2. **Add more channel-specific troubleshooting** for other channel types
3. **Include troubleshooting examples** for common agent issues
4. **Add visual diagrams** for architecture-related troubleshooting
5. **Create video tutorials** for using diagnostic commands
6. **Add FAQ section** based on user support tickets

## Conclusion

The troubleshooting guide is now comprehensive and integrated into the CI pipeline. The documentation build produces both mdBook site and Rust API docs in a single artifact, making it easy to deploy complete documentation for Aisopod.
