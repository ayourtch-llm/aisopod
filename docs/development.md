# Development Guide

This document describes the development workflow for aisopod.

## Prerequisites

- **Rust**: Rust 1.75+ (see [rust-lang.org](https://rust-lang.org))
- **Node.js**: Node.js 20+
- **pnpm**: Install with `npm install -g pnpm`

## Building the Project

### Production Build

To build the entire project including the UI:

```bash
# Build the UI and embed it in the binary
cd ui/ && pnpm build && cd ..

# Build the Rust binary with UI embedded
AISOPOD_BUILD_UI=1 cargo build --release
```

The UI will be built and placed in `web-ui/dist/`, then embedded into the binary.

### Development Build (Skip UI)

For rapid development without rebuilding the UI:

```bash
# Build without UI (UI not embedded)
cargo build
```

Or explicitly skip UI build:

```bash
NO_BUILD_UI=1 cargo build
```

## UI Development

### Prerequisites
- Node.js 20+
- pnpm (`npm install -g pnpm`)

### Hot Reload Development

Run the aisopod gateway and Vite dev server in separate terminals:

**Terminal 1 — Gateway:**
```bash
cargo run
```

**Terminal 2 — Vite Dev Server:**
```bash
cd ui/
pnpm install
pnpm dev
```

Open `http://localhost:5173` in your browser. Changes to UI source
files will hot-reload instantly. API and WebSocket requests are
proxied to the gateway on port 18789.

### Production Build

To build the UI and embed it in the binary:
```bash
cd ui/ && pnpm build && cd ..
AISOPOD_BUILD_UI=1 cargo build --release
```

## Running Tests

Run all tests:

```bash
cargo test
```

Run tests for a specific crate:

```bash
cargo test -p aisopod-gateway
```

Run tests with coverage:

```bash
cargo tarpaulin
```

## Code Style

- Rust: Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- TypeScript: Follow TypeScript best practices
- Run `cargo fmt` before committing
- Run `cargo clippy` to check for common issues

## Directory Structure

```
.
├── crates/              # Rust crates
│   ├── aisopod/         # Main CLI application
│   ├── aisopod-agent/   # Agent execution engine
│   ├── aisopod-channel/ # Channel abstraction layer
│   ├── aisopod-config/  # Configuration management
│   ├── aisopod-gateway/ # HTTP/WebSocket gateway
│   └── ...
├── ui/                  # Web UI (Vite + Lit)
│   ├── src/
│   ├── public/
│   ├── index.html
│   └── vite.config.ts
└── web-ui/              # Embedded UI (generated)
    └── dist/            # Build output
```

## Configuration

Configuration files are located at:

- `~/.config/aisopod/config.toml` (user-specific)
- `aisopod-config.toml` (project-specific)

See `aisopod config generate` for creating a default configuration.
