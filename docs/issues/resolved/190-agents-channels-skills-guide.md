# Issue 190: Write User Guide for Agents, Channels, and Skills

## Summary
Create a comprehensive user guide covering the three core domain concepts of Aisopod — agents, channels, and skills — including setup guides for each Tier 1 channel, agent lifecycle management, and skill reference documentation.

## Location
- Crate: N/A (documentation)
- File: `docs/book/src/agents-channels-skills.md`

## Current Behavior
Agents, channels, and skills are implemented in code but lack user-facing documentation. Users have no guidance on how to create agents, connect channels, or manage skills without reading source code.

## Expected Behavior
A thorough guide at `docs/book/src/agents-channels-skills.md` that explains concepts, provides step-by-step setup instructions for each Tier 1 channel, and documents all available skills — enabling users to configure Aisopod for their specific messaging needs.

## Impact
Agents, channels, and skills are the primary user-facing abstractions. Without clear documentation, users cannot leverage Aisopod's multi-agent, multi-channel capabilities, which is the project's core value proposition.

## Suggested Implementation

1. **Create** `docs/book/src/agents-channels-skills.md` with the following sections:

2. **Agent Concepts section:**
   ```markdown
   ## Agents

   ### What is an Agent?
   An agent is an LLM-backed conversational entity. Each agent has:
   - A **name** (unique identifier)
   - A **model** (which LLM to use)
   - A **system prompt** (personality and instructions)
   - An optional **fallback model** (if primary model fails)
   - A set of **skills** (tools the agent can invoke)

   ### Agent Lifecycle
   1. **Creation** — defined in `config.toml` under `[[agents]]` or via CLI
   2. **Activation** — starts when the gateway loads or on first message
   3. **Message handling** — receives messages, generates responses
   4. **Deactivation** — stops when the gateway shuts down

   ### Creating an Agent

   Via config file:
   \```toml
   [[agents]]
   name = "research-assistant"
   model = "gpt-4o"
   system_prompt = """
   You are a research assistant. Help users find and summarize
   information from the web. Always cite your sources.
   """
   fallback_model = "gpt-3.5-turbo"
   skills = ["web_search", "summarize"]
   \```

   Via CLI:
   \```bash
   aisopod agent create \
     --name research-assistant \
     --model gpt-4o \
     --system-prompt "You are a research assistant." \
     --skills web_search,summarize
   \```

   ### Model Selection and Fallback
   \```toml
   [[agents]]
   name = "resilient-bot"
   model = "claude-3-opus"
   fallback_model = "gpt-4o"
   # If Claude is unavailable or rate-limited, falls back to GPT-4o
   \```

   ### System Prompt Customization
   System prompts support:
   - Multi-line strings (TOML `"""` syntax)
   - Variable interpolation: `{user_name}`, `{channel_name}`
   - File references: `system_prompt_file = "prompts/researcher.md"`
   ```

3. **Channel Setup Guides section** — one subsection per Tier 1 channel:
   ```markdown
   ## Channels

   ### Telegram Setup
   1. Create a bot via [@BotFather](https://t.me/BotFather)
   2. Copy the bot token
   3. Add to config:
      \```toml
      [[channels]]
      type = "telegram"
      token = "${AISOPOD_TELEGRAM_TOKEN}"
      agent = "default"
      \```
   4. Start the gateway: `aisopod gateway start`
   5. Message your bot on Telegram

   ### Discord Setup
   1. Create an application at [Discord Developer Portal](https://discord.com/developers)
   2. Create a bot user and copy the token
   3. Invite the bot to your server with message permissions
   4. Add to config:
      \```toml
      [[channels]]
      type = "discord"
      token = "${AISOPOD_DISCORD_TOKEN}"
      agent = "default"
      guild_ids = ["123456789"]  # optional: restrict to specific servers
      \```
   5. Start the gateway: `aisopod gateway start`

   ### WhatsApp Setup
   1. Set up a WhatsApp Business API account
   2. Configure webhook URL to point to your Aisopod gateway
   3. Add to config:
      \```toml
      [[channels]]
      type = "whatsapp"
      phone_number_id = "..."
      access_token = "${AISOPOD_WHATSAPP_TOKEN}"
      verify_token = "your-verify-token"
      agent = "default"
      \```
   4. Verify the webhook connection

   ### Slack Setup
   1. Create a Slack app at [api.slack.com](https://api.slack.com/apps)
   2. Enable Socket Mode or configure Event Subscriptions URL
   3. Add bot token scopes: `chat:write`, `app_mentions:read`, `im:history`
   4. Install to workspace and copy Bot User OAuth Token
   5. Add to config:
      \```toml
      [[channels]]
      type = "slack"
      bot_token = "${AISOPOD_SLACK_BOT_TOKEN}"
      app_token = "${AISOPOD_SLACK_APP_TOKEN}"  # for Socket Mode
      agent = "default"
      \```
   ```

4. **Agent Binding section:**
   ```markdown
   ### Agent Binding (Routing Channels to Agents)

   Each channel binds to exactly one agent via the `agent` field:
   \```toml
   [[channels]]
   type = "telegram"
   token = "..."
   agent = "research-assistant"   # binds to the agent named "research-assistant"
   \```

   Multiple channels can bind to the same agent. Different channels can
   bind to different agents for specialized behavior.
   ```

5. **Skills Reference section:**
   ```markdown
   ## Skills

   ### Available Skills

   | Skill          | Description                        | Default |
   |----------------|------------------------------------|---------|
   | `web_search`   | Search the web via configured provider | off  |
   | `code_exec`    | Execute code in sandboxed environment  | off  |
   | `summarize`    | Summarize long texts or URLs           | on   |
   | `memory`       | Persistent memory across sessions      | off  |
   | `image_gen`    | Generate images via DALL-E / SD        | off  |
   | `file_read`    | Read uploaded files                    | on   |

   ### Enabling / Disabling Skills
   \```toml
   [[agents]]
   name = "safe-bot"
   model = "gpt-4o"
   skills = ["summarize", "file_read"]  # only these skills enabled
   \```

   ### Creating Custom Skills
   See the [Developer Guide](./developer-guide.md) for implementing custom
   skills via the `Skill` trait.
   ```

6. **Update `SUMMARY.md`** to link to this page.

## Dependencies
- Issue 187 (documentation infrastructure)
- Issue 106 (Tier 1 channels implemented — Telegram, Discord, WhatsApp, Slack)
- Issue 123 (skills system implemented)

## Acceptance Criteria
- [ ] `docs/book/src/agents-channels-skills.md` exists and is linked from `SUMMARY.md`
- [ ] Agent concepts, lifecycle, creation, and management are documented
- [ ] Model selection and fallback configuration are explained
- [ ] System prompt customization (multi-line, variables, file refs) is documented
- [ ] Each Tier 1 channel (Telegram, Discord, WhatsApp, Slack) has a step-by-step setup guide
- [ ] Agent binding (routing channels to agents) is explained
- [ ] Complete skills reference table with enable/disable instructions
- [ ] `mdbook build` succeeds with this page included

## Resolution

Implementation completed on 2026-02-27:

### What Was Implemented
Created comprehensive user guide for agents, channels, and skills:

**1. Agent Concepts Documentation:**
- Defined what an agent is (LLM-backed conversational entity)
- Documented agent lifecycle (creation, activation, message handling, deactivation)
- Explained agent configuration options (name, model, system prompt, fallback model, skills)

**2. Agent Creation Methods:**
- Documented config file approach with `[[agents]]` array
- Explained CLI approach with `aisopod agent create`
- Covered model selection and fallback configuration
- Documented system prompt customization (multi-line, variables, file references)

**3. Channel Setup Guides:**
- Created step-by-step setup for Telegram (BotFather integration)
- Created step-by-step setup for Discord (Developer Portal integration)
- Created step-by-step setup for WhatsApp (Business API configuration)
- Created step-by-step setup for Slack (Socket Mode and Event Subscriptions)

**4. Agent Binding Documentation:**
- Explained how channels bind to agents via `agent` field
- Documented one-to-one and many-to-one channel-to-agent relationships

**5. Skills Reference:**
- Created complete table of available skills (web_search, code_exec, summarize, memory, image_gen, file_read)
- Documented enable/disable methods in agent configuration
- Provided instructions for creating custom skills with link to developer guide

**6. Documentation Linking:**
- Updated SUMMARY.md with agents-channels-skills.md link

### Files Created/Modified
- docs/book/src/agents-channels-skills.md

### Test Results
- mdbook build docs/book: PASSED
- cargo build: PASSED

### Resolution Date
2026-02-27
