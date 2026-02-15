# Shared Utilities

**Crate:** `aisopod-shared`

## Overview

The shared crate provides cross-cutting utilities used by multiple subsystems.
It contains no domain-specific logic of its own — only foundational helpers that
would otherwise be duplicated across crates.

## Key Modules

### Config Evaluation (`config_eval`)

- **Path resolution** — Expand `~`, resolve relative paths against config file
  location.
- **Binary detection** — Check whether a required binary exists on `$PATH`.
- **`$PATH` resolution** — Locate executables with platform-aware search.

### Requirements Validation (`requirements`)

- Validate system prerequisites before startup: required binaries, environment
  variables, OS compatibility checks.
- Returns structured error reports listing all unmet requirements at once.

### Chat Envelope (`chat_envelope`)

- Strip/attach metadata (timestamps, routing info) from messages before they
  reach the AI model.
- Ensures models see clean content without internal transport fields.

### Usage Aggregates (`usage_aggregates`)

- `TokenUsage` — Input/output token counts per request.
- Aggregation helpers for per-session and per-agent cumulative usage tracking.
- Serialization support for usage reporting APIs.

### Frontmatter Parsing (`frontmatter`)

- Parse YAML frontmatter from markdown text (delimited by `---`).
- Used for skill and prompt files that embed metadata in frontmatter blocks.

### Device Auth (`device_auth`)

- Device code flow utilities (RFC 8628) for headless authentication.
- Polling helpers for token exchange after user completes browser-based auth.

## Dependencies

This crate intentionally has minimal dependencies:
- `serde` / `serde_json` — Serialization.
- `anyhow` / `thiserror` — Error handling.
- `chrono` — Timestamps in usage and envelope types.

It depends on **no other aisopod crates**, making it a leaf in the dependency graph.

## Design Decisions

- **Leaf crate with no intra-workspace deps:** Prevents circular dependencies and
  keeps compile times fast — every other crate can depend on `aisopod-shared`
  without pulling in the world.
- **No domain logic:** If a utility is specific to one subsystem, it belongs in
  that subsystem's crate, not here.
- **Structured requirement errors:** Returning all unmet requirements at once
  (rather than failing on the first) gives operators a complete picture in a
  single startup attempt.
