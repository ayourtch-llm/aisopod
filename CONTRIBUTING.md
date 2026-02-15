# Contributing to aisopod

Thank you for your interest in contributing to aisopod! This document provides guidelines and instructions for contributing to this project.

## Table of Contents

- [Prerequisites](#prerequisites)
- [Development Setup](#development-setup)
- [Code Style Guidelines](#code-style-guidelines)
- [Branch Naming Conventions](#branch-naming-conventions)
- [Pull Request Process](#pull-request-process)
- [Testing Requirements](#testing-requirements)

## Prerequisites

Before you begin contributing to aisopod, ensure you have the following installed:

- **Rust stable** (latest) - [Install Rust](https://www.rust-lang.org/tools/install)
- **Cargo** - Comes with Rust installation
- **Git** - [Install Git](https://git-scm.com/book/en/v2/Getting-Started-Installing-Git)

You can verify your installations:

```bash
rustc --version
cargo --version
git --version
```

## Development Setup

1. **Clone the repository**

```bash
git clone https://github.com/your-org/aisopod.git
cd aisopod
```

2. **Build the project**

```bash
cargo build
```

3. **Run tests**

```bash
cargo test
```

4. **Build and run the application**

```bash
cargo run
```

## Code Style Guidelines

### Formatting

All code must be formatted using `rustfmt`:

```bash
cargo fmt
```

You can check formatting without making changes:

```bash
cargo fmt --check
```

### Linting

Run `clippy` to catch common Rust mistakes and improve code quality:

```bash
cargo clippy
```

### Coding Standards

- Follow Rust API Guidelines
- Use descriptive variable and function names
- Add doc comments for all public items
- Use `thiserror` for error types
- Prefer `anyhow::Result` for application-level errors
- Use `tracing` for logging instead of `println!`

## Branch Naming Conventions

Use the following prefixes for branch names:

- `feature/` - New features
- `fix/` - Bug fixes
- `docs/` - Documentation changes
- `test/` - Test additions or modifications
- `refactor/` - Code refactoring
- `chore/` - Maintenance tasks

Examples:

- `feature/add-telegram-channel`
- `fix/session-leak`
- `docs/update-readme`
- `test/add-agent-tests`

## Pull Request Process

1. **Create a feature branch**

```bash
git checkout -b feature/your-feature-name
```

2. **Make your changes**

3. **Run tests**

```bash
cargo test
```

4. **Format and lint your code**

```bash
cargo fmt
cargo clippy
```

5. **Commit your changes**

Use conventional commit format:

- `feat: add new feature`
- `fix: fix bug in session management`
- `docs: update README`
- `test: add unit tests for tools`

6. **Push your branch**

```bash
git push origin feature/your-feature-name
```

7. **Open a Pull Request**

- Provide a clear description of your changes
- Reference any related issues
- Include testing steps if applicable

## Testing Requirements

### All tests must pass

Before submitting a pull request, ensure all tests pass:

```bash
cargo test --all
```

### Add tests for new features

- Unit tests for new functionality
- Integration tests for system-level behavior
- Test edge cases and error conditions

### Test locations

- Unit tests: `src/lib.rs` or `src/main.rs`
- Integration tests: `tests/integration.rs`
- Crate-specific tests: `crates/*/tests/`

### Running tests

```bash
# Run all tests
cargo test

# Run tests for a specific crate
cargo test -p aisopod-agent

# Run tests with coverage
cargo tarpaulin

# Run specific test
cargo test test_function_name
```

## Issue Tracker

We use GitHub Issues to track all bugs and feature requests. Before starting work on a new feature, check if there's an existing issue or create one.

### Issue Templates

Use the appropriate template when creating issues:

- Bug Report - For reporting bugs
- Feature Request - For new features
- Question - For questions about usage or architecture

## Questions?

If you have questions about contributing:

- Open an issue for discussion
- Check existing issues and PRs
- Review the architecture documentation in `docs/`

Thank you for contributing to aisopod!
