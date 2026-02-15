# aisopod

A modular, extensible Rust framework for building AI-powered applications with pluggable LLM providers, tools, and plugins.

## Features / Goals

- **Modular Architecture**: Built as a workspace of focused, interoperable crates
- **LLM Provider Abstraction**: Support for multiple LLM providers through unified interfaces
- **Tool System**: Extensible tool framework for agent operations
- **Plugin System**: Dynamic plugin loading and management
- **Session Management**: Robust conversation and state management
- **Memory System**: Context and memory management for long-term knowledge
- **Gateway API**: RESTful API gateway for external integrations
- **Cross-Platform**: Works on Linux, macOS, and Windows

## Architecture Overview

Aisopod is organized as a Rust workspace with the following crates:

| Crate | Purpose |
|-------|---------|
| `aisopod` | Binary entry point - main application executable |
| `aisopod-shared` | Shared utilities and common types used across all crates |
| `aisopod-config` | Configuration management and loading |
| `aisopod-provider` | LLM provider abstractions and implementations |
| `aisopod-tools` | Tool-use framework for agent operations |
| `aisopod-session` | Session management and conversation state |
| `aisopod-memory` | Memory and context management |
| `aisopod-agent` | Core agent orchestration and decision-making |
| `aisopod-channel` | Communication channels and message passing |
| `aisopod-plugin` | Plugin system and dynamic loading |
| `aisopod-gateway` | RESTful API gateway for external access |

```
┌─────────────────────────────────────────────────────────────────┐
│                    aisopod (Binary)                             │
└─────────────────────────────────────────────────────────────────┘
                              │
        ┌─────────────────────┼─────────────────────┐
        │                     │                     │
        ▼                     ▼                     ▼
┌───────────────┐     ┌───────────────┐     ┌───────────────┐
│ aisopod-config│     │aisopod-shared │     │aisopod-gateway│
└───────────────┘     └───────────────┘     └───────────────┘
        │                     │                     │
        └─────────────────────┼─────────────────────┘
                              │
        ┌─────────────────────┼─────────────────────┐
        │                     │                     │
        ▼                     ▼                     ▼
┌───────────────┐     ┌───────────────┐     ┌───────────────┐
│aisopod-provider│   │aisopod-session│   │aisopod-agent  │
└───────────────┘     └───────────────┘     └───────────────┘
        │                     │                     │
        └─────────────────────┼─────────────────────┘
                              │
        ┌─────────────────────┼─────────────────────┐
        │                     │                     │
        ▼                     ▼                     ▼
┌───────────────┐     ┌───────────────┐     ┌───────────────┐
│aisopod-tools  │     │aisopod-memory │     │aisopod-channel│
└───────────────┘     └───────────────┘     └───────────────┘
                              │
                              ▼
                    ┌───────────────┐
                    │aisopod-plugin │
                    └───────────────┘
```

## Build Instructions

### Prerequisites

- Rust stable (latest)
- Cargo (comes with Rust)
- Git

### Building

```bash
# Clone the repository
git clone https://github.com/your-org/aisopod.git
cd aisopod

# Build the project
cargo build --release

# Run all tests
cargo test

# Run the main application
cargo run --release
```

### Development

```bash
# Format code
cargo fmt

# Run linter
cargo clippy

# Build all crates
cargo build

# Test all crates
cargo test --all

# Run a specific crate's tests
cargo test -p aisopod-agent
```

## License

This project is licensed under the MIT License. See the `LICENSE` file for details.
