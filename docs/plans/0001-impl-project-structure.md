# 0001 — Project Structure & Build System

**Master Plan Reference:** Section 3.1 — Project Structure & Build System  
**Phase:** 1 (Foundation)  
**Dependencies:** None (first implementation step)

---

## Objective

Set up the Rust workspace, crate structure, CI/CD pipeline, and development tooling
that will serve as the foundation for all subsequent implementation work.

---

## Deliverables

### 1. Cargo Workspace Root

Create the top-level `Cargo.toml` workspace definition:

```toml
[workspace]
resolver = "2"
members = [
    "crates/*",
]

[workspace.package]
version = "0.1.0"
edition = "2024"
license = "MIT"
repository = "https://github.com/ayourtch-llm/aisopod"

[workspace.dependencies]
# Shared dependencies pinned at workspace level
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
anyhow = "1"
thiserror = "2"
tracing = "0.1"
tracing-subscriber = "0.3"
```

### 2. Initial Crate Scaffolding

Create skeleton crates with minimal `Cargo.toml` and `lib.rs`/`main.rs`:

| Crate                 | Type    | Purpose                              |
|-----------------------|---------|--------------------------------------|
| `aisopod`             | binary  | Main CLI & gateway entry point       |
| `aisopod-config`      | library | Configuration types & validation     |
| `aisopod-gateway`     | library | HTTP + WebSocket server              |
| `aisopod-agent`       | library | Agent execution engine               |
| `aisopod-provider`    | library | Provider trait abstractions           |
| `aisopod-channel`     | library | Channel trait abstractions            |
| `aisopod-tools`       | library | Agent tool implementations           |
| `aisopod-plugin`      | library | Plugin system                        |
| `aisopod-session`     | library | Session management                   |
| `aisopod-memory`      | library | Memory/vector DB                     |
| `aisopod-shared`      | library | Shared utilities                     |

Each crate should contain:
- `Cargo.toml` with workspace dependency references
- `src/lib.rs` (or `src/main.rs` for binary) with module stub
- Basic doc comments explaining the crate's purpose

### 3. CI/CD Pipeline

Set up GitHub Actions workflows:

- **Build & Test:** `cargo build --workspace`, `cargo test --workspace`
- **Lint:** `cargo clippy --workspace -- -D warnings`
- **Format check:** `cargo fmt --all -- --check`
- **Security audit:** `cargo audit`

### 4. Development Tooling

- `.rustfmt.toml` — Formatting configuration
- `.clippy.toml` — Clippy configuration (if needed)
- `rust-toolchain.toml` — Pin Rust toolchain version
- `.gitignore` — Rust-specific ignores (`/target`, etc.)
- `Makefile` or `justfile` — Common development commands

### 5. Documentation Skeleton

- `README.md` — Project overview, build instructions, architecture
- `docs/` directory structure initialized
- `CONTRIBUTING.md` — Development setup and contribution guide

---

## Acceptance Criteria

- [ ] `cargo build --workspace` succeeds with no errors
- [ ] `cargo test --workspace` passes (even if no tests yet)
- [ ] `cargo clippy --workspace` produces no warnings
- [ ] `cargo fmt --all -- --check` passes
- [ ] CI pipeline runs on push/PR
- [ ] All crates resolve dependencies correctly
- [ ] README contains build and development instructions
