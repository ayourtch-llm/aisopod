# Issue 189: Write User Guide for Configuration

## Summary
Create a comprehensive configuration reference guide that documents every configuration option, environment variable, and file format used by Aisopod, with working examples for common setups.

## Location
- Crate: N/A (documentation)
- File: `docs/book/src/configuration.md`

## Current Behavior
Configuration options are defined in code but lack user-facing documentation. Users must read source code, struct definitions, or scattered comments to understand what can be configured and how.

## Expected Behavior
A complete configuration reference at `docs/book/src/configuration.md` that serves as the single source of truth for all Aisopod configuration — file format, every section, every field, every environment variable — with annotated examples and common-setup recipes.

## Impact
Configuration is the second thing a user touches after installation. Incomplete or unclear configuration docs lead to misconfiguration, wasted time, and support requests. This guide is essential for self-service adoption.

## Suggested Implementation

1. **Create** `docs/book/src/configuration.md` with the following sections:

2. **Config File Format section:**
   ```markdown
   ## Configuration File Format

   Aisopod uses **TOML** as its primary configuration format. JSON5 is also
   supported for backward compatibility.

   Default config location:
   - Linux/macOS: `~/.config/aisopod/config.toml`
   - Docker: `/config/config.toml`

   Override with: `aisopod --config /path/to/config.toml`
   or: `AISOPOD_CONFIG=/path/to/config.toml`
   ```

3. **Environment Variables Reference section** — a complete table:
   ```markdown
   ## Environment Variables

   All environment variables use the `AISOPOD_` prefix.

   | Variable                    | Description                        | Default       |
   |-----------------------------|------------------------------------|---------------|
   | `AISOPOD_CONFIG`            | Path to config file                | `~/.config/aisopod/config.toml` |
   | `AISOPOD_LOG`               | Log level (trace/debug/info/warn/error) | `info`   |
   | `AISOPOD_HOST`              | Gateway listen address             | `127.0.0.1`  |
   | `AISOPOD_PORT`              | Gateway listen port                | `3080`       |
   | `AISOPOD_AUTH_TOKEN`        | Bearer token for API auth          | (none)       |
   | `AISOPOD_OPENAI_API_KEY`    | OpenAI API key                     | (none)       |
   | `AISOPOD_ANTHROPIC_API_KEY` | Anthropic API key                  | (none)       |
   | ...                         | ...                                | ...          |

   Environment variables **override** config file values.
   ```

4. **Config Sections Explained** — one subsection per top-level key:
   ```markdown
   ## Config Sections

   ### `[server]` — Gateway Server
   \```toml
   [server]
   host = "0.0.0.0"
   port = 3080
   workers = 4            # number of async worker threads
   request_timeout = 30   # seconds
   \```

   ### `[auth]` — Authentication
   \```toml
   [auth]
   mode = "token"         # token | password | device | none
   token = "sk-..."       # or use AISOPOD_AUTH_TOKEN env var
   \```

   ### `[agents]` — Agent Definitions
   \```toml
   [[agents]]
   name = "default"
   model = "gpt-4o"
   system_prompt = "You are a helpful assistant."
   fallback_model = "gpt-3.5-turbo"
   skills = ["web_search", "code_exec"]
   \```

   ### `[models]` — Model Provider Configuration
   \```toml
   [models.openai]
   api_key = "${AISOPOD_OPENAI_API_KEY}"
   base_url = "https://api.openai.com/v1"

   [models.anthropic]
   api_key = "${AISOPOD_ANTHROPIC_API_KEY}"
   \```

   ### `[channels]` — Channel Integrations
   \```toml
   [[channels]]
   type = "telegram"
   token = "${AISOPOD_TELEGRAM_TOKEN}"
   agent = "default"
   \```

   ### `[tools]` — Tool / Skill Configuration
   \```toml
   [tools.sandbox]
   enabled = true
   runtime = "docker"
   timeout = 30
   \```
   ```

5. **Example Configurations section** — complete, copy-pasteable configs:
   ```markdown
   ## Example Configurations

   ### Minimal Setup (single agent, no channels)
   \```toml
   [auth]
   mode = "none"

   [[agents]]
   name = "assistant"
   model = "gpt-4o"

   [models.openai]
   api_key = "sk-..."
   \```

   ### Production Setup (multi-agent, Telegram + Discord)
   \```toml
   [server]
   host = "0.0.0.0"
   port = 3080

   [auth]
   mode = "token"
   token = "${AISOPOD_AUTH_TOKEN}"

   [[agents]]
   name = "general"
   model = "gpt-4o"
   system_prompt = "You are a general-purpose assistant."
   skills = ["web_search"]

   [[agents]]
   name = "coder"
   model = "claude-3-opus"
   system_prompt = "You are a coding assistant."
   skills = ["code_exec"]

   [[channels]]
   type = "telegram"
   token = "${AISOPOD_TELEGRAM_TOKEN}"
   agent = "general"

   [[channels]]
   type = "discord"
   token = "${AISOPOD_DISCORD_TOKEN}"
   agent = "coder"

   [models.openai]
   api_key = "${AISOPOD_OPENAI_API_KEY}"

   [models.anthropic]
   api_key = "${AISOPOD_ANTHROPIC_API_KEY}"
   \```
   ```

6. **Update `SUMMARY.md`** to ensure the configuration page is linked.

## Dependencies
- Issue 187 (documentation infrastructure must be in place)
- Issue 025 (config tests confirm the supported format and field names)

## Acceptance Criteria
- [ ] `docs/book/src/configuration.md` exists and is linked from `SUMMARY.md`
- [ ] TOML and JSON5 formats are documented
- [ ] Complete environment variables table with all `AISOPOD_*` variables
- [ ] Every config section (`server`, `auth`, `agents`, `models`, `channels`, `tools`) is documented with field descriptions
- [ ] At least two complete example configurations (minimal and production)
- [ ] Examples are syntactically valid TOML
- [ ] `mdbook build` succeeds with this page included

---
*Created: 2026-02-15*
