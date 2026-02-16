# Issue 188: Write Getting Started Guide

## Summary
Create a comprehensive getting-started guide that enables a new user to install Aisopod, complete onboarding, and send their first message within 10 minutes. This is the single most important documentation page for adoption.

## Location
- Crate: N/A (documentation)
- File: `docs/book/src/getting-started.md`

## Current Behavior
No structured getting-started documentation exists. New users must read source code or scattered notes to understand how to install and run Aisopod.

## Expected Behavior
A polished, step-by-step guide at `docs/book/src/getting-started.md` covering system requirements, all installation methods, first-run onboarding, quick-start messaging, and an architecture overview — all written for someone with no prior Aisopod experience.

## Impact
The getting-started guide is the entry point for every new user. A clear, fast onboarding experience directly drives adoption and reduces support burden. Without this, users will abandon the project before experiencing its value.

## Suggested Implementation

1. **Create the file** `docs/book/src/getting-started.md` with these sections:

2. **System Requirements section:**
   ```markdown
   ## System Requirements

   | Component       | Minimum           | Recommended        |
   |-----------------|-------------------|--------------------|
   | OS              | Linux, macOS, Windows (WSL2) | Linux or macOS |
   | Rust toolchain  | 1.75+             | latest stable      |
   | Memory          | 512 MB            | 2 GB               |
   | Disk            | 100 MB            | 500 MB             |
   | Network         | Required for LLM API calls | Broadband     |

   You will also need an API key from at least one supported LLM provider
   (OpenAI, Anthropic, etc.).
   ```

3. **Installation Methods section** with tabs/subsections:
   ```markdown
   ## Installation

   ### Pre-built Binary (Recommended)
   Download the latest release for your platform:
   \```bash
   curl -fsSL https://aisopod.dev/install.sh | sh
   \```

   ### Homebrew (macOS / Linux)
   \```bash
   brew install aisopod/tap/aisopod
   \```

   ### Cargo Install (from source)
   \```bash
   cargo install aisopod
   \```

   ### Docker
   \```bash
   docker run -it --rm -v ~/.config/aisopod:/config ghcr.io/aisopod/aisopod:latest
   \```

   Verify installation:
   \```bash
   aisopod --version
   \```
   ```

4. **First-Run Onboarding Wizard section:**
   ```markdown
   ## First Run

   Run the onboarding wizard:
   \```bash
   aisopod init
   \```

   The wizard will guide you through:
   1. Choosing your default LLM provider
   2. Entering your API key (stored securely in your OS keychain)
   3. Creating your first agent
   4. Optionally connecting a messaging channel

   Your configuration is saved to `~/.config/aisopod/config.toml`.
   ```

5. **Quick Start: Send Your First Message:**
   ```markdown
   ## Quick Start

   Send a message directly from the terminal:
   \```bash
   aisopod chat "Hello, what can you do?"
   \```

   Or start the gateway and connect via the REST API:
   \```bash
   aisopod gateway start
   curl http://localhost:3080/v1/chat/completions \
     -H "Content-Type: application/json" \
     -d '{"messages": [{"role": "user", "content": "Hello!"}]}'
   \```
   ```

6. **Architecture Overview section:**
   ```markdown
   ## Architecture Overview

   Aisopod is organized around three core concepts:

   - **Gateway**: The HTTP/WebSocket server that receives messages and routes them.
   - **Agent**: An LLM-backed conversational entity with a system prompt,
     model selection, and bound skills.
   - **Channel**: An integration with an external messaging platform
     (Telegram, Discord, Slack, etc.).

   \```
   Channel → Gateway → Agent → LLM Provider
                ↕
             Skills / Tools
   \```

   See the [Developer Guide](./developer-guide.md) for a deep dive into
   the crate structure.
   ```

7. **Update `SUMMARY.md`** to link to this page (should already be linked from Issue 187).

## Dependencies
- Issue 187 (documentation infrastructure must be in place)
- Issue 125 (gateway command must be functional for quick-start examples)

## Acceptance Criteria
- [ ] `docs/book/src/getting-started.md` exists and is linked from `SUMMARY.md`
- [ ] System requirements table is accurate and complete
- [ ] All four installation methods are documented with working commands
- [ ] Onboarding wizard walkthrough matches actual `aisopod init` behavior
- [ ] Quick-start example sends a message and receives a response
- [ ] Architecture overview introduces gateway, agent, and channel concepts
- [ ] A new user can follow the guide and have Aisopod running in under 10 minutes
- [ ] `mdbook build` succeeds with this page included

---
*Created: 2026-02-15*
