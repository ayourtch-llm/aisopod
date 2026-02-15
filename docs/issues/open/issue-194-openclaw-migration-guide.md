# Issue 194: Write OpenClaw to Aisopod Migration Guide

## Summary
Create a detailed migration guide that helps existing OpenClaw users transition to Aisopod, covering configuration format changes, environment variable mapping, feature parity, breaking changes, and data migration.

## Location
- Crate: N/A (documentation)
- File: `docs/book/src/migration-guide.md`

## Current Behavior
No migration documentation exists. OpenClaw users considering Aisopod have no guidance on how to transition their existing setup, what has changed, or what migration steps are required.

## Expected Behavior
A thorough migration guide at `docs/book/src/migration-guide.md` that an existing OpenClaw user can follow to migrate their entire setup — config, environment, data — to Aisopod with minimal disruption.

## Impact
Migration documentation is critical for user retention during the OpenClaw → Aisopod transition. Without it, existing users face uncertainty and may abandon the migration, fragmenting the user base.

## Suggested Implementation

1. **Create** `docs/book/src/migration-guide.md` with the following sections:

2. **Overview section:**
   ```markdown
   ## Migration Overview

   Aisopod is the successor to OpenClaw, rebuilt from the ground up in Rust.
   This guide helps you migrate your existing OpenClaw installation.

   **What's new:**
   - Rust-native binary (no Node.js runtime required)
   - TOML configuration (replacing JSON5)
   - WebSocket support alongside REST
   - Plugin system with sandboxed execution
   - Multi-agent architecture

   **Migration steps:**
   1. Install Aisopod
   2. Convert your configuration
   3. Update environment variables
   4. Migrate session data (optional)
   5. Update deployment scripts
   6. Verify functionality
   ```

3. **Configuration Format Migration section:**
   ```markdown
   ## Configuration Migration

   OpenClaw uses JSON5; Aisopod uses TOML. Use the automated migration
   tool or convert manually.

   ### Automated Migration
   \```bash
   aisopod migrate --from-openclaw /path/to/openclaw/config.json5
   \```
   This generates an equivalent `config.toml`.

   ### Manual Mapping

   **OpenClaw JSON5:**
   \```json5
   {
     server: { port: 3000 },
     auth: { type: "bearer", token: "sk-..." },
     models: {
       default: { provider: "openai", model: "gpt-4" }
     },
     channels: [
       { type: "telegram", token: "bot-token" }
     ]
   }
   \```

   **Aisopod TOML equivalent:**
   \```toml
   [server]
   port = 3080  # note: default port changed from 3000 to 3080

   [auth]
   mode = "token"  # "type" → "mode", "bearer" → "token"
   token = "sk-..."

   [[agents]]
   name = "default"
   model = "gpt-4"

   [models.openai]
   api_key = "sk-..."

   [[channels]]
   type = "telegram"
   token = "bot-token"
   agent = "default"  # new: channels must bind to an agent
   \```
   ```

4. **Environment Variable Mapping section:**
   ```markdown
   ## Environment Variable Mapping

   All environment variables have been renamed from `OPENCLAW_*` to `AISOPOD_*`.

   | OpenClaw Variable            | Aisopod Variable              | Notes                |
   |------------------------------|-------------------------------|----------------------|
   | `OPENCLAW_PORT`              | `AISOPOD_PORT`                | Default: 3000 → 3080 |
   | `OPENCLAW_HOST`              | `AISOPOD_HOST`                | Unchanged behavior   |
   | `OPENCLAW_AUTH_TOKEN`        | `AISOPOD_AUTH_TOKEN`          |                      |
   | `OPENCLAW_OPENAI_KEY`        | `AISOPOD_OPENAI_API_KEY`     | Suffix changed       |
   | `OPENCLAW_ANTHROPIC_KEY`     | `AISOPOD_ANTHROPIC_API_KEY`  | Suffix changed       |
   | `OPENCLAW_TELEGRAM_TOKEN`    | `AISOPOD_TELEGRAM_TOKEN`     |                      |
   | `OPENCLAW_DISCORD_TOKEN`     | `AISOPOD_DISCORD_TOKEN`      |                      |
   | `OPENCLAW_LOG_LEVEL`         | `AISOPOD_LOG`                | Name shortened       |
   | `OPENCLAW_DATA_DIR`          | `AISOPOD_DATA_DIR`           |                      |
   | `OPENCLAW_CONFIG`            | `AISOPOD_CONFIG`             |                      |

   **Quick rename script:**
   \```bash
   # In your .env or deployment scripts, replace:
   sed -i 's/OPENCLAW_/AISOPOD_/g' .env
   sed -i 's/OPENAI_KEY/OPENAI_API_KEY/g' .env
   sed -i 's/ANTHROPIC_KEY/ANTHROPIC_API_KEY/g' .env
   sed -i 's/LOG_LEVEL/LOG/g' .env
   \```
   ```

5. **Feature Parity Checklist section:**
   ```markdown
   ## Feature Parity

   | Feature                  | OpenClaw | Aisopod | Notes                          |
   |--------------------------|----------|---------|--------------------------------|
   | Chat completions API     | ✅       | ✅      | Same endpoint path             |
   | Streaming responses      | ✅       | ✅      | Now also via WebSocket         |
   | Telegram channel         | ✅       | ✅      |                                |
   | Discord channel          | ✅       | ✅      |                                |
   | WhatsApp channel         | ✅       | ✅      |                                |
   | Slack channel            | ❌       | ✅      | New in Aisopod                 |
   | WebSocket API            | ❌       | ✅      | New in Aisopod                 |
   | Multi-agent              | ❌       | ✅      | New in Aisopod                 |
   | Plugin system            | ❌       | ✅      | New in Aisopod                 |
   | Session memory           | ✅       | ✅      | Storage format changed         |
   | Code execution sandbox   | ✅       | ✅      | Now uses Docker/Wasm           |
   | Web search               | ✅       | ✅      |                                |
   | Custom system prompts    | ✅       | ✅      | File reference support added   |
   ```

6. **Breaking Changes and Workarounds section:**
   ```markdown
   ## Breaking Changes

   ### Default Port Changed
   - **Old:** `3000`
   - **New:** `3080`
   - **Workaround:** Set `AISOPOD_PORT=3000` or update clients

   ### Channel Agent Binding Required
   - **Old:** Channels auto-route to default model
   - **New:** Each channel must explicitly set `agent = "name"`
   - **Workaround:** Create a `default` agent and bind all channels to it

   ### Auth Config Key Renamed
   - **Old:** `auth.type = "bearer"`
   - **New:** `auth.mode = "token"`

   ### Session Storage Format Changed
   - **Old:** JSON files in `~/.openclaw/sessions/`
   - **New:** SQLite database at `~/.local/share/aisopod/sessions.db`
   - **Workaround:** Use `aisopod migrate --sessions` (see Data Migration)
   ```

7. **Data Migration section:**
   ```markdown
   ## Data Migration

   ### Session History
   \```bash
   aisopod migrate --sessions --from ~/.openclaw/sessions/
   \```
   This imports OpenClaw session JSON files into Aisopod's SQLite store.

   ### Memory / Knowledge Base
   \```bash
   aisopod migrate --memory --from ~/.openclaw/memory/
   \```

   ### Verification
   After migration, verify:
   \```bash
   aisopod doctor --check-data
   \```
   ```

8. **Update `SUMMARY.md`** to link to this page.

## Dependencies
- Issue 187 (documentation infrastructure)
- Issue 161 (migration utility implemented — `aisopod migrate` command)

## Acceptance Criteria
- [ ] `docs/book/src/migration-guide.md` exists and is linked from `SUMMARY.md`
- [ ] Configuration format migration documented with side-by-side JSON5/TOML examples
- [ ] Complete environment variable mapping table
- [ ] Feature parity checklist covers all OpenClaw features
- [ ] All breaking changes listed with workarounds
- [ ] Data migration steps documented (sessions, memory)
- [ ] Automated migration command (`aisopod migrate`) is documented
- [ ] `mdbook build` succeeds with this page included

---
*Created: 2026-02-15*
