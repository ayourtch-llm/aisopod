# Issue 187: Set Up Documentation Infrastructure with mdBook

## Summary
Install and configure mdBook (or similar Rust-native documentation tool) as the primary documentation framework for the Aisopod project. Create the foundational directory structure, configuration, and CI integration so that all subsequent documentation issues have a consistent build pipeline.

## Location
- Crate: N/A (project-wide tooling)
- File: `docs/book/book.toml`, `docs/book/src/SUMMARY.md`

## Current Behavior
No structured documentation site exists. Project information is scattered across READMEs and inline code comments with no unified, browsable documentation output.

## Expected Behavior
A fully configured mdBook setup that:
- Lives under `docs/book/` with `book.toml` and `src/SUMMARY.md`
- Supports theme customization, full-text search, and dark/light mode toggle
- Builds a static HTML documentation site via `mdbook build`
- Integrates into the CI pipeline so documentation builds are validated on every push

## Impact
This is the foundational issue for all documentation work in Plan 0019. Every subsequent documentation issue (188–196) depends on this infrastructure being in place. Without it, there is no consistent way to author, preview, or publish project documentation.

## Suggested Implementation

1. **Install mdBook** — add it as a development dependency or CI tool:
   ```bash
   cargo install mdbook
   ```

2. **Create the directory structure:**
   ```bash
   mkdir -p docs/book/src
   ```

3. **Create `docs/book/book.toml`:**
   ```toml
   [book]
   authors = ["Aisopod Contributors"]
   language = "en"
   multilingual = false
   src = "src"
   title = "Aisopod Documentation"

   [build]
   build-dir = "build"

   [output.html]
   default-theme = "light"
   preferred-dark-theme = "ayu"
   git-repository-url = "https://github.com/AIsopod/aisopod"
   edit-url-template = "https://github.com/AIsopod/aisopod/edit/main/docs/book/{path}"

   [output.html.search]
   enable = true
   limit-results = 30
   use-hierarchical-headings = true
   ```

4. **Create `docs/book/src/SUMMARY.md`:**
   ```markdown
   # Summary

   [Introduction](./introduction.md)

   # User Guide

   - [Getting Started](./getting-started.md)
   - [Configuration](./configuration.md)
   - [Agents, Channels & Skills](./agents-channels-skills.md)

   # Reference

   - [CLI Command Reference](./cli-reference.md)
   - [REST & WebSocket API](./api-reference.md)

   # Developer Guide

   - [Architecture & Contributing](./developer-guide.md)
   - [Migration from OpenClaw](./migration-guide.md)

   # Operations

   - [Security & Deployment](./security-deployment.md)
   - [Troubleshooting](./troubleshooting.md)
   ```

5. **Create a placeholder `docs/book/src/introduction.md`:**
   ```markdown
   # Introduction

   Welcome to the Aisopod documentation.

   Aisopod is an AI-native messaging gateway...
   ```

6. **Add a CI step** (e.g., in `.github/workflows/ci.yml`):
   ```yaml
   docs:
     name: Build Documentation
     runs-on: ubuntu-latest
     steps:
       - uses: actions/checkout@v4
       - name: Install mdBook
         run: cargo install mdbook --version 0.4.37 --locked
       - name: Build docs
         run: mdbook build docs/book
       - name: Upload artifact
         uses: actions/upload-artifact@v4
         with:
           name: documentation
           path: docs/book/build
   ```

7. **Verify locally:**
   ```bash
   cd docs/book && mdbook build && mdbook serve
   ```
   Open `http://localhost:3000` and confirm the site renders with search, theme toggle, and correct structure.

## Dependencies
- Issue 014 (CI workflow must exist to add the docs build step)

## Acceptance Criteria
- [ ] `docs/book/book.toml` exists with correct configuration
- [ ] `docs/book/src/SUMMARY.md` defines the documentation structure
- [ ] `mdbook build docs/book` completes without errors
- [ ] Built site includes search functionality
- [ ] Dark/light mode toggle works
- [ ] CI pipeline includes a documentation build step that passes

---
*Created: 2026-02-15*
